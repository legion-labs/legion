use std::sync::{Arc, Mutex};

use lgn_ecs::prelude::{Commands, Entity, Query};
use lgn_input::{
    mouse::{MouseButton, MouseButtonInput, MouseMotion},
    ElementState,
};
use lgn_math::Vec2;
use lgn_transform::prelude::Transform;

use crate::{
    components::{ManipulatorComponent, PickedComponent},
    render_pass::PickingData,
};

use super::ManipulatorType;

pub struct PickingIdBlock {
    picking_ids: Vec<u32>,
    entity_ids: Vec<u64>,
    base_picking_id: u32,
}

impl PickingIdBlock {
    pub fn new(base_picking_id: u32, block_size: u32) -> Self {
        let mut generation_counts = Vec::with_capacity(block_size as usize);
        generation_counts.reserve(block_size as usize);
        for i in 0..block_size {
            generation_counts.push(base_picking_id + i as u32);
        }

        Self {
            picking_ids: generation_counts,
            entity_ids: vec![0; block_size as usize],
            base_picking_id,
        }
    }

    pub fn aquire_picking_id(&mut self, entity: Entity) -> Option<u32> {
        let picking_id = self.picking_ids.pop();
        if let Some(picking_id) = picking_id {
            let index = (picking_id & 0x00FFFFFF) - self.base_picking_id;
            self.entity_ids[index as usize] = entity.to_bits();
            Some(picking_id)
        } else {
            None
        }
    }

    pub fn release_picking_id(&mut self, picking_id: u32) {
        let generation = (picking_id >> 24) + 1;
        let picking_id = picking_id & 0x00FFFFFF;
        assert!(picking_id >= self.base_picking_id);

        let index = picking_id - self.base_picking_id;
        assert!(index < self.entity_ids.len() as u32);

        self.entity_ids[index as usize] = 0;
        let picking_id = (generation << 24) | picking_id;
        self.picking_ids.push(picking_id);
    }

    pub fn entity_id_for_picking_id(&self, picking_id: u32) -> Entity {
        let picking_id = picking_id & 0x00FFFFFF;
        assert!(picking_id >= self.base_picking_id);

        let index = picking_id - self.base_picking_id;
        assert!(index < self.entity_ids.len() as u32);

        Entity::from_bits(self.entity_ids[index as usize])
    }

    pub fn base_picking_id(&self) -> u32 {
        self.base_picking_id
    }
}

#[derive(Clone, PartialEq)]
pub(crate) enum PickingState {
    Ready,
    Rendering,
    Waiting,
    Processing,
    Completed,
}

pub struct PickingManagerInner {
    block_size: u32,
    picking_blocks: Vec<Option<PickingIdBlock>>,
    mouse_input: MouseButtonInput,
    screen_rect: Vec2,
    manip_entity_base_transform: Transform,
    picking_state: PickingState,
    current_cpu_frame_no: u64,
    picked_cpu_frame_no: u64,
    picked_pos: Vec2,
    current_picking_data: Vec<PickingData>,
    current_type: ManipulatorType,
    manipulated_entity: Entity,
}

#[derive(Clone)]
pub struct PickingManager {
    inner: Arc<Mutex<PickingManagerInner>>,
}

impl PickingManager {
    pub fn new(block_size: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PickingManagerInner {
                block_size,
                picking_blocks: Vec::new(),
                mouse_input: MouseButtonInput {
                    button: MouseButton::Left,
                    state: ElementState::Released,
                    pos: Vec2::NAN,
                },
                manip_entity_base_transform: Transform::default(),
                screen_rect: Vec2::default(),
                picking_state: PickingState::Ready,
                current_cpu_frame_no: 0,
                picked_cpu_frame_no: u64::MAX,
                picked_pos: Vec2::ZERO,
                current_picking_data: Vec::new(),
                current_type: ManipulatorType::Position,
                manipulated_entity: Entity::new(u32::MAX),
            })),
        }
    }

    pub fn aquire_picking_id_block(&self) -> PickingIdBlock {
        let mut inner = self.inner.lock().unwrap();

        let mut most_free = 0;
        let mut most_free_idx = inner.picking_blocks.len();

        for i in 0..inner.picking_blocks.len() {
            if let Some(block) = &inner.picking_blocks[i] {
                let free_count = block.picking_ids.len();

                if free_count > most_free {
                    most_free = free_count;
                    most_free_idx = i;
                }
            }
        }
        if most_free_idx < inner.picking_blocks.len() {
            return inner.picking_blocks[most_free_idx].take().unwrap();
        }

        let result = PickingIdBlock::new(
            inner.picking_blocks.len() as u32 * inner.block_size,
            inner.block_size,
        );
        inner.picking_blocks.push(None);
        result
    }

    pub fn release_picking_id_block(&self, block: PickingIdBlock) {
        let inner = &mut *self.inner.lock().unwrap();

        let block_id = block.base_picking_id() / inner.block_size;
        assert!(inner.picking_blocks[block_id as usize].is_none());

        inner.picking_blocks[block_id as usize] = Some(block);
    }

    pub fn release_picking_ids(&mut self, picking_ids: &[u32]) {
        let inner = &mut *self.inner.lock().unwrap();

        for picking_id in picking_ids {
            let base_id = picking_id & 0x00FFFFFF;
            let block_id = base_id / inner.block_size as u32;

            if let Some(block) = &mut inner.picking_blocks[block_id as usize] {
                block.release_picking_id(*picking_id);
            } else {
                panic!();
            }
        }
    }

    pub fn frame_no_picked(&self) -> u64 {
        let inner = self.inner.lock().unwrap();

        inner.picked_cpu_frame_no
    }

    pub fn frame_no_for_picking(&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();

        inner.picking_state = PickingState::Waiting;
        inner.picked_cpu_frame_no
    }

    pub(crate) fn picking_state(&self) -> PickingState {
        let inner = self.inner.lock().unwrap();

        inner.picking_state.clone()
    }

    pub fn mouse_button_down(&self) -> bool {
        let inner = self.inner.lock().unwrap();

        inner.mouse_input.state.is_pressed()
    }

    pub fn current_cursor_pos(&self) -> Vec2 {
        let inner = self.inner.lock().unwrap();

        inner.mouse_input.pos
    }

    pub fn picked_pos(&self) -> Vec2 {
        let inner = self.inner.lock().unwrap();

        inner.picked_pos
    }

    pub fn set_mouse_button_input(&self, input: &MouseButtonInput) {
        let inner = &mut *self.inner.lock().unwrap();

        if input.button == MouseButton::Left {
            inner.mouse_input = input.clone();

            inner.current_cpu_frame_no += 1;
            if inner.picking_state == PickingState::Ready || inner.mouse_input.state.is_pressed() {
                inner.picked_cpu_frame_no = inner.current_cpu_frame_no;
                inner.picking_state = PickingState::Rendering;
                inner.picked_pos = inner.mouse_input.pos;
            }

            if inner.picking_state == PickingState::Completed
                && !inner.mouse_input.state.is_pressed()
            {
                inner.picking_state = PickingState::Ready;
            }
        }
    }

    pub fn set_mouse_moition_event(&self, input: &MouseMotion) {
        let inner = &mut *self.inner.lock().unwrap();

        if inner.mouse_input.state.is_pressed() && inner.mouse_input.button == MouseButton::Left {
            inner.mouse_input.pos += input.delta / 1.33;
        }
    }

    pub fn screen_rect(&self) -> Vec2 {
        let inner = self.inner.lock().unwrap();

        inner.screen_rect
    }

    pub fn set_screen_rect(&self, screen_rect: Vec2) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.screen_rect = screen_rect;
    }

    pub(crate) fn set_picked(&self, picked_data_set: &[PickingData]) {
        let inner = &mut *self.inner.lock().unwrap();

        for picking_data in picked_data_set {
            if !inner
                .current_picking_data
                .iter()
                .any(|existing_data| existing_data.picking_id == picking_data.picking_id)
            {
                inner.current_picking_data.push(*picking_data);
            }
        }
        if inner.picking_state == PickingState::Waiting {
            inner.picking_state = PickingState::Processing;
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(super) fn update_picking_components(
        &self,
        mut commands: Commands<'_, '_>,
        mut picked_components: Query<
            '_,
            '_,
            (
                Entity,
                &Transform,
                &mut PickedComponent,
                Option<&ManipulatorComponent>,
            ),
        >,
        manipulator_entities: Query<'_, '_, (Entity, &ManipulatorComponent)>,
    ) {
        let inner = &mut *self.inner.lock().unwrap();

        if inner.picking_state == PickingState::Processing {
            if inner.current_picking_data.is_empty() {
                inner.manipulated_entity = Entity::new(u32::MAX);
            }

            let mut picked_entities = Vec::with_capacity(inner.current_picking_data.len());
            for picking_data in &inner.current_picking_data {
                let base_id = picking_data.picking_id & 0x00FFFFFF;
                let block_id = base_id / inner.block_size as u32;

                if let Some(block) = &mut inner.picking_blocks[block_id as usize] {
                    picked_entities.push(block.entity_id_for_picking_id(picking_data.picking_id));
                }
            }
            let mut manipulator_picked = false;
            for picked_entity in &picked_entities {
                for (entity, _manipulator) in manipulator_entities.iter() {
                    if entity == *picked_entity {
                        manipulator_picked = true;
                    }
                }
            }

            for (entity, transform, mut picked_component, manipulator_component) in
                picked_components.iter_mut()
            {
                if !manipulator_picked || manipulator_component.is_some() {
                    picked_component.replace_picking_ids(
                        entity,
                        &mut inner.current_picking_data,
                        &mut picked_entities,
                    );

                    if manipulator_component.is_none() && !picked_component.is_empty() {
                        inner.manip_entity_base_transform = *transform;
                        inner.manipulated_entity = entity;
                    }
                }
            }

            let i = 0;
            while i < inner.current_picking_data.len() {
                let entity_id = picked_entities[i];
                let mut is_manipulator = false;

                for (entity, _manipulator) in manipulator_entities.iter() {
                    if entity == entity_id {
                        is_manipulator = true;
                    }
                }

                if !manipulator_picked || is_manipulator {
                    let mut add_component = commands.get_or_spawn(entity_id);
                    let mut new_component = PickedComponent::new();
                    new_component.replace_picking_ids(
                        picked_entities[i],
                        &mut inner.current_picking_data,
                        &mut picked_entities,
                    );
                    add_component.insert(new_component);
                } else {
                    inner.current_picking_data.swap_remove(i);
                    picked_entities.swap_remove(i);
                }
            }
            inner.picking_state = PickingState::Completed;
        }
    }

    pub fn manipulator_type(&self) -> ManipulatorType {
        let inner = self.inner.lock().unwrap();

        inner.current_type
    }

    pub fn manipulated_entity(&self) -> Entity {
        let inner = self.inner.lock().unwrap();

        inner.manipulated_entity
    }

    pub fn set_manip_entity(&self, entity: Entity, base_transform: &Transform) {
        let mut inner = self.inner.lock().unwrap();

        inner.manip_entity_base_transform = *base_transform;
        inner.manipulated_entity = entity;
    }

    pub fn base_picking_transform(&self) -> Transform {
        let inner = self.inner.lock().unwrap();

        inner.manip_entity_base_transform
    }
}
