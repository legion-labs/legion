use super::{default_compilerid, CompilerInfo, CompilerInput};
use crate::{CompiledAsset, Error};
use legion_assets::{AssetId, AssetType};
use legion_resources::ResourceType;

fn compile(input: &mut CompilerInput<'_>) -> Result<Vec<CompiledAsset>, Error> {
    // todo: convert ResourceId to AssetId
    let guid = AssetId::new(AssetType::Texture, 2);

    let asset_content = input
        .project
        .read_resource(input.resource)
        .map_err(|_e| Error::NotFound)?;

    let compiled_asset = {
        let mut buffer = asset_content;
        buffer.reverse();
        buffer
    };

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
    handled_resources: &[ResourceType::Texture],
    code_id: 1,
    data_id: 1,
    compilerid_func: default_compilerid,
    compile_func: compile,
};
