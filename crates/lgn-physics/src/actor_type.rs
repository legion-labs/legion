use crate::RigidActorType;

use crate::runtime::{
    PhysicsRigidBox, PhysicsRigidCapsule, PhysicsRigidConvexMesh, PhysicsRigidHeightField,
    PhysicsRigidPlane, PhysicsRigidSphere, PhysicsRigidTriangleMesh,
};

pub(crate) trait WithActorType {
    fn get_actor_type(&self) -> RigidActorType;
}

impl WithActorType for PhysicsRigidBox {
    fn get_actor_type(&self) -> RigidActorType {
        self.actor_type
    }
}

impl WithActorType for PhysicsRigidCapsule {
    fn get_actor_type(&self) -> RigidActorType {
        self.actor_type
    }
}

impl WithActorType for PhysicsRigidConvexMesh {
    fn get_actor_type(&self) -> RigidActorType {
        self.actor_type
    }
}

impl WithActorType for PhysicsRigidHeightField {
    fn get_actor_type(&self) -> RigidActorType {
        self.actor_type
    }
}

impl WithActorType for PhysicsRigidPlane {
    fn get_actor_type(&self) -> RigidActorType {
        self.actor_type
    }
}

impl WithActorType for PhysicsRigidSphere {
    fn get_actor_type(&self) -> RigidActorType {
        self.actor_type
    }
}

impl WithActorType for PhysicsRigidTriangleMesh {
    fn get_actor_type(&self) -> RigidActorType {
        self.actor_type
    }
}
