use super::{default_compilerid, CompilerInfo, CompilerInput};
use crate::{
    test_resource::{NullResource, NullResourceProc, RESOURCE_TEXTURE},
    CompiledAsset, Error,
};
use legion_assets::{AssetId, AssetType};
use legion_resources::ResourceRegistry;

// This is just a test compiler. Normally following asset types should be defined by relevant modules.
//const RESOURCE_TEXTURE: ResourceType = ResourceType::new(b"texture");
const ASSET_TEXTURE: AssetType = AssetType::new(b"texture");

fn compile(input: &mut CompilerInput<'_>) -> Result<Vec<CompiledAsset>, Error> {
    let mut resources = ResourceRegistry::default();
    resources.register_type(RESOURCE_TEXTURE, Box::new(NullResourceProc {}));

    // todo: convert ResourceId to AssetId
    let guid = AssetId::new(ASSET_TEXTURE, 2);

    let resource = input
        .project
        .load_resource(input.resource, &mut resources)?;
    let resource = resource.get::<NullResource>(&resources).unwrap();

    let compiled_asset = {
        let mut content = resource.content.as_bytes().to_owned();
        content.reverse();
        content
    };

    // todo: create Asset and serialize it.

    let checksum = input
        .asset_store
        .store(&compiled_asset)
        .ok_or(Error::IOError)?;

    let asset = CompiledAsset {
        guid,
        checksum,
        size: compiled_asset.len(),
    };
    Ok(vec![asset])
}

pub static COMPILER_INFO: CompilerInfo = CompilerInfo {
    handled_resources: &[RESOURCE_TEXTURE],
    code_id: 1,
    data_id: 1,
    compilerid_func: default_compilerid,
    compile_func: compile,
};
