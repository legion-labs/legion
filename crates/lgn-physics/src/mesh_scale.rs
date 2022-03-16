// TODO: contribute this code to physx-rs crate

use physx::{
    prelude::{PxQuat, PxVec3},
    traits::Class,
};
use physx_sys::{PxMeshScale, PxMeshScale_new, PxMeshScale_new_3};

use crate::runtime;

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

// Copy and Clone are not derived by data-gen for struct used in components
impl Copy for runtime::MeshScale {}

#[allow(clippy::expl_impl_clone_on_copy)]
impl Clone for runtime::MeshScale {
    fn clone(&self) -> Self {
        Self {
            scale: self.scale,
            rotation: self.rotation,
        }
    }
}

impl From<runtime::MeshScale> for PxMeshScale {
    fn from(value: runtime::MeshScale) -> Self {
        let scale: PxVec3 = value.scale.into();
        let rotation: PxQuat = value.rotation.into();
        Self::new(&scale, &rotation)
    }
}
