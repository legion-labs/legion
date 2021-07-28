use super::entity::ComponentType;
use super::reflection::Reference;
#[cfg(test)]
use crate::prelude::*;
use std::marker::PhantomData;

pub struct System<F, Args>
where
    F: SignatureAnalyzer<Args>,
{
    _name: &'static str,
    signature: Vec<ComponentAccess>,
    functor: F,
    args: PhantomData<Args>,
}

impl<F, Args> System<F, Args>
where
    F: SignatureAnalyzer<Args>,
{
    pub fn new(name: &'static str, functor: F) -> Self {
        let mut system = Self {
            _name: name,
            signature: Vec::new(),
            functor,
            args: PhantomData,
        };
        system.build_signature();
        system
    }

    fn build_signature(&mut self) {
        self.functor.add_component_accesses(&mut self.signature);
    }
}

#[derive(Debug)]
pub struct ComponentAccess {
    component_type: ComponentType,
    mutable: bool,
}

impl ComponentAccess {
    fn new<T>() -> Self
    where
        T: Reference,
    {
        Self {
            component_type: ComponentType::new::<T>(),
            mutable: T::is_mutable(),
        }
    }
}

pub enum SystemError {}

pub type SystemResult = Result<(), SystemError>;

pub trait SignatureAnalyzer<Args> {
    fn add_component_accesses(&self, signature: &mut Vec<ComponentAccess>);
}

impl<F> SignatureAnalyzer<()> for F
where
    F: Fn() -> SystemResult,
{
    fn add_component_accesses(&self, _signature: &mut Vec<ComponentAccess>) {}
}

impl<F, Args> SignatureAnalyzer<Args> for F
where
    F: Fn(Args) -> SystemResult,
    Args: Reference,
{
    fn add_component_accesses(&self, signature: &mut Vec<ComponentAccess>) {
        signature.push(ComponentAccess::new::<Args>());
    }
}

impl<F, Args0, Args1> SignatureAnalyzer<(Args0, Args1)> for F
where
    F: Fn(Args0, Args1) -> SystemResult,
    Args0: Reference,
    Args1: Reference,
{
    fn add_component_accesses(&self, signature: &mut Vec<ComponentAccess>) {
        signature.push(ComponentAccess::new::<Args0>());
        signature.push(ComponentAccess::new::<Args1>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_nothing() -> SystemResult {
        Ok(())
    }

    #[test]
    fn build_system_no_args() {
        let system = System::new("do_nothing", do_nothing);
        println!("signature: {:?}", system.signature);
        assert_eq!(system.signature.len(), 0);
    }

    struct Position(Vector3);

    fn read_position(_pos: &Position) -> SystemResult {
        Ok(())
    }

    fn drift_position(pos: &mut Position) -> SystemResult {
        pos.0.x += 0.001;
        Ok(())
    }

    #[test]
    fn build_system_single_arg() {
        let immutable_system = System::new("read_position", read_position);
        println!("signature: {:?}", immutable_system.signature);
        assert_eq!(immutable_system.signature.len(), 1);
        let immutable_arg = &immutable_system.signature[0];
        assert!(!immutable_arg.mutable);

        let mutable_system = System::new("drift_position", drift_position);
        println!("signature: {:?}", mutable_system.signature);
        assert_eq!(mutable_system.signature.len(), 1);
        let mutable_arg = &mutable_system.signature[0];
        assert!(mutable_arg.mutable);

        assert_eq!(
            immutable_arg.component_type.get_name(),
            mutable_arg.component_type.get_name()
        );
        assert_eq!(
            immutable_arg.component_type.get_id(),
            mutable_arg.component_type.get_id()
        );
    }

    struct Velocity(Vector3);

    fn update_position(pos: &mut Position, vel: &Velocity) -> SystemResult {
        pos.0.x += vel.0.x;
        pos.0.y += vel.0.y;
        pos.0.z += vel.0.z;
        Ok(())
    }

    #[test]
    fn build_system_with_two_args() {
        let system = System::new("update_position", &update_position);
        println!("signature: {:?}", system.signature);
        assert_eq!(system.signature.len(), 2);
        assert_eq!(system.signature[0].component_type.get_name(), "Position");
        assert!(system.signature[0].mutable);
        assert_eq!(system.signature[1].component_type.get_name(), "Velocity");
        assert!(!system.signature[1].mutable);
    }
}
