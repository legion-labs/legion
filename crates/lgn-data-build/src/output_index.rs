use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::Seek,
    path::{Path, PathBuf},
};

use lgn_content_store::Checksum;
use lgn_data_compiler::CompiledResource;
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::ResourceTypeAndId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CompiledResourceInfo {
    /// The path the resource was compiled from, i.e.:
    /// "ResourcePathId("anim.fbx").push("anim.offline")
    pub(crate) compile_path: ResourcePathId,
    pub(crate) context_hash: AssetHash,
    pub(crate) source_hash: AssetHash,
    /// The path the resource was compiled into, i.e.:
    /// "ResourcePathId("anim.fbx").push("anim.offline")["idle"]
    pub(crate) compiled_path: ResourcePathId,
    pub(crate) compiled_checksum: Checksum,
    pub(crate) compiled_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CompiledResourceReference {
    pub(crate) compile_path: ResourcePathId,
    pub(crate) context_hash: AssetHash,
    pub(crate) source_hash: AssetHash,
    pub(crate) compiled_path: ResourcePathId,
    pub(crate) compiled_reference: ResourcePathId,
}

impl CompiledResourceReference {
    pub fn is_same_context(&self, resource_info: &CompiledResourceInfo) -> bool {
        self.context_hash == resource_info.context_hash
            && self.source_hash == resource_info.source_hash
    }

    pub fn is_from_same_source(&self, resource_info: &CompiledResourceInfo) -> bool {
        self.is_same_context(resource_info) && self.compile_path == resource_info.compile_path
    }

    pub fn is_reference_of(&self, resource_info: &CompiledResourceInfo) -> bool {
        self.is_from_same_source(resource_info) && self.compiled_path == resource_info.compiled_path
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
struct OutputContent {
    version: String,
    compiled_resources: Vec<CompiledResourceInfo>,
    compiled_resource_references: Vec<CompiledResourceReference>,
    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    pathid_mapping: BTreeMap<ResourceTypeAndId, ResourcePathId>,
}

impl OutputContent {
    // sort contents so serialization is deterministic
    fn pre_serialize(&mut self) {
        self.compiled_resources.sort_by(|a, b| {
            let mut result = a.compile_path.cmp(&b.compile_path);
            if result == Ordering::Equal {
                result = a.compiled_path.cmp(&b.compiled_path);
            }
            result
        });
        self.compiled_resource_references.sort_by(|a, b| {
            let mut result = a.compile_path.cmp(&b.compile_path);
            if result == Ordering::Equal {
                result = a.compiled_path.cmp(&b.compiled_path);
                if result == Ordering::Equal {
                    result = a.compiled_reference.cmp(&b.compiled_reference);
                }
            }
            result
        });
    }
}

#[derive(Debug)]
pub(crate) struct OutputIndex {
    content: OutputContent,
    file: File,
}

impl OutputIndex {
    pub(crate) fn create_new(output_index: &Path, version: &str) -> Result<Self, Error> {
        let output_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(output_index)
            .map_err(|e| Error::Io(e.into()))?;

        let output_content = OutputContent {
            version: String::from(version),
            compiled_resources: vec![],
            compiled_resource_references: vec![],
            pathid_mapping: BTreeMap::<_, _>::new(),
        };

        serde_json::to_writer_pretty(&output_file, &output_content)
            .map_err(|e| Error::Io(e.into()))?;

        Ok(Self {
            content: output_content,
            file: output_file,
        })
    }

    fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let output_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())
            .map_err(|_e| Error::NotFound)?;

        let output_content: OutputContent =
            serde_json::from_reader(&output_file).map_err(|e| Error::Io(e.into()))?;

        Ok(Self {
            content: output_content,
            file: output_file,
        })
    }

    pub(crate) fn open(output_index: &Path, version: &str) -> Result<Self, Error> {
        if !output_index.exists() {
            return Err(Error::NotFound);
        }

        let output_index = Self::load(output_index)?;

        if output_index.content.version != version {
            return Err(Error::VersionMismatch {
                value: output_index.content.version,
                expected: version.to_owned(),
            });
        }

        Ok(output_index)
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.content.pre_serialize();
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.file, &self.content).map_err(|e| Error::Io(e.into()))?;
        Ok(())
    }

    pub(crate) fn insert_compiled(
        &mut self,
        compile_path: &ResourcePathId,
        context_hash: u64,
        source_hash: u64,
        compiled_resources: &[CompiledResource],
        compiled_references: &[(ResourcePathId, ResourcePathId)],
    ) {
        // For now we assume there is not concurrent compilation
        // so there is no way to compile the same resources twice.
        // Once we support it we will have to make sure the result of the compilation
        // is exactly the same for all compiled_assets.
        assert!(self
            .find_compiled(compile_path, context_hash, source_hash)
            .is_none());

        let mut compiled_assets_desc: Vec<_> = compiled_resources
            .iter()
            .map(|asset| CompiledResourceInfo {
                compile_path: compile_path.clone(),
                context_hash: context_hash.into(),
                source_hash: source_hash.into(),
                compiled_path: asset.path.clone(),
                compiled_checksum: asset.checksum,
                compiled_size: asset.size,
            })
            .collect();

        let mut compiled_references_desc: Vec<_> = compiled_references
            .iter()
            .map(
                |(compiled_guid, compiled_reference)| CompiledResourceReference {
                    context_hash: context_hash.into(),
                    compile_path: compile_path.clone(),
                    source_hash: source_hash.into(),
                    compiled_path: compiled_guid.clone(),
                    compiled_reference: compiled_reference.clone(),
                },
            )
            .collect();

        self.content
            .compiled_resources
            .append(&mut compiled_assets_desc);

        self.content
            .compiled_resource_references
            .append(&mut compiled_references_desc);
    }

    pub(crate) fn find_compiled(
        &self,
        compile_path: &ResourcePathId,
        context_hash: u64,
        source_hash: u64,
    ) -> Option<(Vec<CompiledResourceInfo>, Vec<CompiledResourceReference>)> {
        let asset_objects: Vec<CompiledResourceInfo> = self
            .content
            .compiled_resources
            .iter()
            .filter(|asset| {
                &asset.compile_path == compile_path
                    && asset.context_hash.get() == context_hash
                    && asset.source_hash.get() == source_hash
            })
            .cloned()
            .collect();

        if asset_objects.is_empty() {
            None
        } else {
            let asset_references: Vec<CompiledResourceReference> = self
                .content
                .compiled_resource_references
                .iter()
                .filter(|reference| {
                    &reference.compile_path == compile_path
                        && reference.context_hash.get() == context_hash
                        && reference.source_hash.get() == source_hash
                })
                .cloned()
                .collect();

            Some((asset_objects, asset_references))
        }
    }

    pub(crate) fn output_index_file(buildindex_dir: impl AsRef<Path>) -> PathBuf {
        buildindex_dir.as_ref().join("output.index")
    }

    pub fn record_pathid(&mut self, id: &ResourcePathId) {
        self.content
            .pathid_mapping
            .insert(id.resource_id(), id.clone());
    }

    pub fn lookup_pathid(&self, id: ResourceTypeAndId) -> Option<ResourcePathId> {
        self.content.pathid_mapping.get(&id).cloned()
    }
}

#[cfg(test)]
mod tests {

    use crate::output_index::OutputIndex;

    #[tokio::test]
    async fn version_check() {
        let work_dir = tempfile::tempdir().unwrap();

        let buildindex_dir = work_dir.path();
        {
            let _output_index =
                OutputIndex::create_new(&OutputIndex::output_index_file(&buildindex_dir), "0.0.1")
                    .unwrap();
        }
        assert!(
            OutputIndex::open(&OutputIndex::output_index_file(&buildindex_dir), "0.0.2").is_err()
        );
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetHash(u64);

impl AssetHash {
    pub(crate) fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for AssetHash {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Serialize for AssetHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_u64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for AssetHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                u64::from_be_bytes(digits.try_into().unwrap())
            } else {
                u64::deserialize(deserializer)?
            }
        };
        Ok(value.into())
    }
}
