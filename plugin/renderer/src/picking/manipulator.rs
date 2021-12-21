//use std::sync::{Arc, Mutex};

trait Manipulator {
    fn new() -> Self;
}

struct PositionManipulator {}

impl Manipulator for PositionManipulator {
    fn new() -> Self {
        Self {}
    }
}

// struct RotationManipulator {}
// struct ScaleManipulator {}

// struct ManipulatorManagerInner {}

// struct ManipulatorManager {
//     inner: Arc<Mutex<ManipulatorManagerInner>>,
// }

//impl ManipulatorManager {}
