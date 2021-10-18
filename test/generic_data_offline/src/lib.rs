use legion_data_offline::resource::ResourceRegistryOptions;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

pub fn register_resource_types(registry: ResourceRegistryOptions) -> ResourceRegistryOptions {
    registry.add_type::<TestEntity>()
}
