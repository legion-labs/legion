//! `DataContainer`

trait OfflineDataContainer {
    fn create_from_json(json_data: &str) -> Self;
    fn write_to_json(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;
    const SIGNATURE_HASH: u64;
}

#[cfg(test)]
mod tests {

    use crate::data_container::OfflineDataContainer;
    pub use legion_data_offline_macros::DataContainer;
    use legion_math::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, DataContainer)]
    #[allow(missing_docs)]
    pub struct Transform {
        pub position: Vec3,
        pub rotation: Quat,
        pub scale: Vec3,
        pub apply_to_children: bool,
    }

    #[derive(Debug, DataContainer)]
    #[allow(dead_code)]
    #[allow(missing_docs)]
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

        let _test = RuntimeEntity {
            ..Default::default()
        };

        let new_entity = Entity::create_from_json(json_data);
        dbg!(&new_entity);

        //let mut file = File::create("d:\\test.json").expect("new file");
        //new_entity.write_to_json(&mut file);
        //_default_instance.write_to_json(&mut file);
    }
}
