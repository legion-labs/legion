use std::any::Any;

pub trait AsAny: Any {
    /// Cast to &dyn Any type.
    fn as_any(&self) -> &dyn Any;

    /// Cast to &mut dyn Any type.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
where
    T: 'static + Sized,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
