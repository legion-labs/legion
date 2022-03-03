// TODO: contribute this code to physx-rs crate

use physx::{
    prelude::{PxQuat, PxVec3},
    traits::Class,
};
use physx_sys::{PxMeshScale, PxMeshScale_new, PxMeshScale_new_3};

impl<T> MeshScale for T where T: Class<PxMeshScale> {}

pub trait MeshScale: Class<PxMeshScale> {
    #[allow(clippy::new_ret_no_self)]
    fn new(scale: &PxVec3, rotation: &PxQuat) -> PxMeshScale {
        #[allow(unsafe_code)]
        unsafe {
            PxMeshScale_new_3(scale.as_ptr(), rotation.as_ptr())
        }
    }

    fn default() -> PxMeshScale {
        #[allow(unsafe_code)]
        unsafe {
            PxMeshScale_new()
        }
    }
}
