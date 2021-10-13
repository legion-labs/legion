use crate::offline_to_runtime::FromOffline;
use legion_data_compiler::compiler_api::{CompilationOutput, CompilerContext, CompilerError};
use legion_data_offline::resource::OfflineResource;
use legion_data_runtime::Resource;
use serde::Serialize;

#[allow(dead_code)]
pub fn compile<OfflineType, RuntimeType>(
    mut context: CompilerContext<'_>,
) -> Result<CompilationOutput, CompilerError>
where
    OfflineType: OfflineResource + 'static,
    RuntimeType: Resource + FromOffline<OfflineType> + Serialize,
{
    let mut resources = context.take_registry().add_loader::<OfflineType>().create();

    let resource = resources.load_sync::<OfflineType>(context.source.content_id());
    let resource = resource.get(&resources).unwrap();

    let asset = RuntimeType::from_offline(resource);
    let compiled_asset = bincode::serialize(&asset).unwrap();

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}
