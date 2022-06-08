use std::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderLayerId(u8);

impl RenderLayerId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

const MAX_RENDER_LAYERS: usize = std::mem::size_of::<RenderLayerMask>() * 8;

#[derive(Clone, PartialEq, Eq)]
pub struct RenderLayer {
    name: String,
    id: RenderLayerId,
}

impl RenderLayer {
    pub fn id(&self) -> RenderLayerId {
        self.id
    }

    pub fn mask(&self) -> u64 {
        1u64 << self.id.0
    }
}

#[derive(Default)]
pub struct RenderLayerBuilder {
    render_layers: Vec<RenderLayer>,
}

impl RenderLayerBuilder {
    pub fn add_render_layer(mut self, name: &str) -> Self {
        let id = self.render_layers.len();
        assert!(id < MAX_RENDER_LAYERS);
        self.render_layers.push(RenderLayer {
            name: name.to_string(),
            id: RenderLayerId(id as u8),
        });
        self
    }

    pub fn finalize(self) -> RenderLayers {
        RenderLayers {
            render_layers: self.render_layers,
        }
    }
}

pub struct RenderLayers {
    render_layers: Vec<RenderLayer>,
}

impl RenderLayers {
    pub fn get_from_name(&self, name: &str) -> &RenderLayer {
        self.try_get_from_name(name).unwrap()
    }

    pub fn try_get_from_name(&self, name: &str) -> Option<&RenderLayer> {
        self.render_layers.iter().find(|l| l.name == name)
    }

    pub fn iter(&self) -> Iter<'_, RenderLayer> {
        self.render_layers.iter()
    }
}

pub const RENDER_LAYER_DEPTH: RenderLayerId = RenderLayerId(0);
pub const RENDER_LAYER_OPAQUE: RenderLayerId = RenderLayerId(1);
pub const RENDER_LAYER_PICKING: RenderLayerId = RenderLayerId(2);

#[derive(Default, Clone, Copy)]
pub struct RenderLayerMask(u64);

impl RenderLayerMask {
    pub fn add(&mut self, render_layer: &RenderLayer) {
        self.0 |= render_layer.mask();
    }

    pub fn iter(self) -> RenderLayerIterator {
        RenderLayerIterator::new(self)
    }
}

impl IntoIterator for &RenderLayerMask {
    type Item = RenderLayerId;

    type IntoIter = RenderLayerIterator;

    fn into_iter(self) -> Self::IntoIter {
        RenderLayerIterator::new(*self)
    }
}

pub struct RenderLayerIterator {
    mask: RenderLayerMask,
}

impl RenderLayerIterator {
    pub fn new(mask: RenderLayerMask) -> Self {
        Self { mask }
    }
}

impl Iterator for RenderLayerIterator {
    type Item = RenderLayerId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask.0 == 0 {
            None
        } else {
            let trailing_zeros = self.mask.0.trailing_zeros();
            self.mask.0 &= !(1 << trailing_zeros);
            Some(RenderLayerId(trailing_zeros as u8))
        }
    }
}
