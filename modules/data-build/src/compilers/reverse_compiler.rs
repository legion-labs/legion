use super::{default_compilerid, CompilerInfo, CompilerInput};
use crate::{CompiledAsset, Error};
use legion_assets::{AssetId, AssetType};
use legion_resources::ResourceType;

fn compile(input: &mut CompilerInput<'_>) -> Result<Vec<CompiledAsset>, Error> {
    // todo: convert ResourceId to AssetId
    let guid = AssetId::new(AssetType::Texture, 2);

    if let Some(asset) = input.asset_store.find(guid) {
        return Ok(vec![asset]);
    }

    let asset_content = input
        .project
        .read_resource(input.resource)
        .map_err(|_e| Error::NotFound)?;

    let compiled_asset = {
        let mut buffer = asset_content;
        buffer.reverse();
        buffer
    };

    let asset = input
        .asset_store
        .store(guid, &compiled_asset)
        .ok_or(Error::IOError)?;
    Ok(vec![asset])
}

pub static COMPILER_INFO: CompilerInfo = CompilerInfo {
    handled_resources: &[ResourceType::Texture],
    code_id: 1,
    data_id: 1,
    compilerid_func: default_compilerid,
    compile_func: compile,
};
