pub trait Identifier: Copy + num_traits::NumAssign {}
impl<T> Identifier for T where T: Copy + num_traits::NumAssign {}

pub struct IdentifierGenerator<T: Identifier> {
    next_valid_id: T,
}

impl<T: Identifier> IdentifierGenerator<T> {
    pub fn get_new_id(&mut self) -> T {
        self.next_valid_id += T::one();
        self.next_valid_id
    }
}

impl<T: Identifier> Default for IdentifierGenerator<T> {
    fn default() -> Self {
        Self {
            next_valid_id: T::zero(),
        }
    }
}
