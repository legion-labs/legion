//! ['DataContainer] Test
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(missing_docs)]
use crate::{asset::*, resource::*};
pub use legion_data_offline_macros::DataContainer;
use legion_math::prelude::*;

#[derive(Debug, DataContainer)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub apply_to_children: bool,
}

#[derive(Debug, DataContainer)]
/// Base Entity
pub struct Entity {
    #[legion(default = "Entity", readonly, category = "Name")]
    name: String,

    #[legion(default = false)]
    test_bool: bool,

    #[legion(default = 42.56, offline)]
    test_float: f32,

    #[legion(default = 123)]
    test_int: i32,

    test_vec3: Vec3,

    test_blob: Vec<u8>,
}

#[test]
fn test_entity_serialization() {
    let json_data = r#"
        {
            "_class" : "Entity",
            "_base" : "AssetPathId",
            "name": "EntityA",
            "test_bool" : true,
            "test_int" : 345345,
            "test_float" : 345.678,
            "test_vec3" : [2,2,2]
        }"#;

    let _default_instance = Entity {
        ..Default::default()
    };

    let test = RuntimeEntity {
        ..Default::default()
    };

    let new_entity = Entity::create_from_json(json_data);
    dbg!(&new_entity);

    //let mut file = File::create("d:\\test.json").expect("new file");
    //new_entity.write_to_json(&mut file);
    //_default_instance.write_to_json(&mut file);
}
