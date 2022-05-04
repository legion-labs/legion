use crossbeam_channel::Sender;
use lgn_ecs::prelude::Component;

/// Component that will broadcast an event with some of its contents when dropped
pub struct ComponentWithDropEvent<C, E>
where
    C: Component,
    E: for<'a> From<&'a C>,
{
    component: C,
    sender: Sender<E>,
}

impl<C, E> Drop for ComponentWithDropEvent<C, E>
where
    C: Component,
    E: for<'a> From<&'a C>,
{
    fn drop(&mut self) {
        let _result = self.sender.send((&self.component).into());
    }
}
