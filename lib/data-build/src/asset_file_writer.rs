use crate::Error;
use byteorder::{LittleEndian, WriteBytesExt};
use legion_content_store::ContentStore;
use legion_data_runtime::{ResourceId, ResourceType};

const ASSET_FILE_VERSION: u16 = 1;

// todo: no asset ids are written because we assume 1 asset in asset_file now.
pub fn write_assetfile<A, R>(
    asset_list: A,
    reference_list: R,
    content_store: &impl ContentStore,
    mut writer: impl std::io::Write,
) -> Result<usize, Error>
where
    A: Iterator<Item = (ResourceId, i128)>,
    R: Iterator<Item = (ResourceId, (ResourceId, ResourceId))>,
    A: Clone,
    R: Clone,
{
    // asset file header

    let mut written = 0;
    writer
        .write_u16::<LittleEndian>(ASSET_FILE_VERSION)
        .map_err(|_e| Error::LinkFailed)?;
    written += std::mem::size_of::<u16>();

    let mut primary_dependencies: Vec<ResourceId> =
        reference_list.into_iter().map(|r| r.1 .0).collect();
    primary_dependencies.dedup();

    writer
        .write_u64::<LittleEndian>(primary_dependencies.len() as u64)
        .map_err(|_e| Error::LinkFailed)?;
    written += std::mem::size_of::<u64>();

    for dep in primary_dependencies {
        writer
            .write_u128::<LittleEndian>(unsafe { std::mem::transmute::<ResourceId, u128>(dep) })
            .map_err(|_e| Error::LinkFailed)?;
        written += std::mem::size_of::<u128>();
    }

    // secion header
    let kind = asset_list.clone().next().unwrap().0.ty();
    writer
        .write_u32::<LittleEndian>(unsafe { std::mem::transmute::<ResourceType, u32>(kind) })
        .map_err(|_e| Error::LinkFailed)?;
    written += std::mem::size_of::<ResourceType>();

    writer
        .write_u64::<LittleEndian>(asset_list.clone().into_iter().count() as u64)
        .map_err(|_e| Error::LinkFailed)?;
    written += std::mem::size_of::<u64>();

    // assets
    for asset in asset_list {
        let source_data = content_store
            .read(asset.1)
            .ok_or(Error::InvalidContentStore)?;

        writer
            .write_u64::<LittleEndian>(source_data.len() as u64)
            .map_err(|_e| Error::LinkFailed)?;
        written += std::mem::size_of::<u64>();

        written += writer.write(&source_data).map_err(|_e| Error::LinkFailed)?;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use byteorder::{LittleEndian, ReadBytesExt};
    use legion_content_store::{ContentStore, RamContentStore};
    use legion_data_runtime::{Resource, ResourceId, ResourceType};

    use crate::asset_file_writer::{write_assetfile, ASSET_FILE_VERSION};

    #[test]
    fn one_asset_no_references() {
        let mut content_store = RamContentStore::default();

        let asset_id = ResourceId::new(refs_asset::RefsAsset::TYPE, 1);
        let reference_list: Vec<(ResourceId, (ResourceId, ResourceId))> = Vec::new();
        let asset_content = b"test_content".to_vec();
        let asset_checksum = content_store.store(&asset_content).expect("to store asset");
        assert_eq!(content_store.read(asset_checksum).unwrap(), asset_content);

        let binary_assetfile = {
            let mut output = vec![];
            let nbytes = write_assetfile(
                std::iter::once((asset_id, asset_checksum)),
                reference_list.iter().cloned(),
                &content_store,
                &mut output,
            )
            .expect("asset file");

            assert_eq!(nbytes, output.len());
            output
        };

        {
            let mut assetfile_reader = &binary_assetfile[..];

            let version = assetfile_reader
                .read_u16::<LittleEndian>()
                .expect("valid data");
            assert_eq!(version, ASSET_FILE_VERSION);

            let primary_reference_count = assetfile_reader
                .read_u64::<LittleEndian>()
                .expect("to read usize");
            assert_eq!(primary_reference_count, 0);

            let asset_type = unsafe {
                std::mem::transmute::<u32, ResourceType>(
                    assetfile_reader
                        .read_u32::<LittleEndian>()
                        .expect("valid data"),
                )
            };
            assert_eq!(asset_type, refs_asset::RefsAsset::TYPE);

            let asset_count = assetfile_reader
                .read_u64::<LittleEndian>()
                .expect("valid data");
            assert_eq!(asset_count, 1);

            let nbytes = assetfile_reader
                .read_u64::<LittleEndian>()
                .expect("valid data");

            let mut content = Vec::new();
            content.resize(nbytes as usize, 0);
            assetfile_reader
                .read_exact(&mut content)
                .expect("valid data");
            assert_eq!(&content, &asset_content);
        }
    }

    #[test]
    fn two_dependent_assets() {
        let mut content_store = RamContentStore::default();

        let child_id = ResourceId::new(refs_asset::RefsAsset::TYPE, 1);
        let child_content = b"child".to_vec();
        let child_checksum = content_store.store(&child_content).expect("to store asset");
        assert_eq!(content_store.read(child_checksum).unwrap(), child_content);

        let parent_id = ResourceId::new(refs_asset::RefsAsset::TYPE, 2);
        let parent_content = b"parent".to_vec();
        let parent_checksum = content_store
            .store(&parent_content)
            .expect("to store asset");
        assert_eq!(content_store.read(parent_checksum).unwrap(), parent_content);

        let reference_list = vec![(parent_id, (child_id, child_id))];

        let parent_assetfile = {
            let mut output = vec![];
            let nbytes = write_assetfile(
                std::iter::once((parent_id, parent_checksum)),
                reference_list.iter().cloned(),
                &content_store,
                &mut output,
            )
            .expect("asset file");

            assert_eq!(nbytes, output.len());
            output
        };

        let _child_assetfile = {
            let mut output = vec![];
            let nbytes = write_assetfile(
                std::iter::once((child_id, child_checksum)),
                std::iter::empty(),
                &content_store,
                &mut output,
            )
            .expect("asset file");

            assert_eq!(nbytes, output.len());
            output
        };

        //println!("{:?} : {:?}", parent_id, parent_assetfile);
        //println!("{:?} : {:?}", child_id, _child_assetfile);

        {
            let mut assetfile_reader = &parent_assetfile[..];

            let version = assetfile_reader
                .read_u16::<LittleEndian>()
                .expect("valid data");
            assert_eq!(version, ASSET_FILE_VERSION);

            let primary_reference_count = assetfile_reader
                .read_u64::<LittleEndian>()
                .expect("to read usize");
            assert_eq!(primary_reference_count, reference_list.len() as u64);

            for (_, (primary_ref, secondary_ref)) in &reference_list {
                let asset_id = unsafe {
                    std::mem::transmute::<u128, ResourceId>(
                        assetfile_reader
                            .read_u128::<LittleEndian>()
                            .expect("read asset id"),
                    )
                };

                assert_eq!(&asset_id, primary_ref);
                assert_eq!(&asset_id, secondary_ref);
            }

            let asset_type = unsafe {
                std::mem::transmute::<u32, ResourceType>(
                    assetfile_reader
                        .read_u32::<LittleEndian>()
                        .expect("valid data"),
                )
            };
            assert_eq!(asset_type, refs_asset::RefsAsset::TYPE);

            let asset_count = assetfile_reader
                .read_u64::<LittleEndian>()
                .expect("valid data");

            assert_eq!(asset_count, 1);

            for _ in 0..asset_count {
                let nbytes = assetfile_reader
                    .read_u64::<LittleEndian>()
                    .expect("valid data");

                let mut content = Vec::new();
                content.resize(nbytes as usize, 0);
                assetfile_reader
                    .read_exact(&mut content)
                    .expect("valid data");
                assert_eq!(&content, &parent_content);
            }
        }
    }
}
