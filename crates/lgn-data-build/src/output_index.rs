use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::Seek,
    path::{Path, PathBuf},
    str::FromStr,
};

use lgn_content_store::Checksum;
use lgn_data_compiler::CompiledResource;
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::ResourceTypeAndId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use sqlx::{migrate::MigrateDatabase, Executor};

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug)]
struct LinkedResource {
    id: ResourcePathId,
    context_hash: AssetHash,
    source_hash: AssetHash,
    checksum: Checksum,
    size: usize,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
struct OutputContent {
    version: String,
    compiled_resources: Vec<CompiledResourceInfo>,
    compiled_resource_references: Vec<CompiledResourceReference>,
    #[serde_as(as = "Vec<(_, _)>")]
    linked_resources: BTreeMap<(ResourcePathId, AssetHash, AssetHash), (Checksum, usize)>,
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
    database: sqlx::AnyPool,
}

impl OutputIndex {
    fn database_uri(output_index: &Path, version: &str) -> String {
        let db_path = output_index
            .parent()
            .unwrap()
            .join(format!("output-{}.db3", version));
        format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"))
    }

    pub(crate) async fn create_new(output_index: &Path, version: &str) -> Result<Self, Error> {
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
            linked_resources: BTreeMap::<_, _>::new(),
            pathid_mapping: BTreeMap::<_, _>::new(),
        };

        serde_json::to_writer_pretty(&output_file, &output_content)
            .map_err(|e| Error::Io(e.into()))?;

        let database = {
            let db_uri = Self::database_uri(output_index, version);
            sqlx::Any::create_database(&db_uri)
                .await
                .map_err(Error::Database)?;
            let connection = sqlx::any::AnyPoolOptions::new()
                .connect(&db_uri)
                .await
                .map_err(Error::Database)?;

            let statement = "
                CREATE TABLE compiled_output(
                    compile_path VARCHAR(255),
                    context_hash BIGINT,
                    source_hash BIGINT,
                    compiled_path VARCHAR(255),
                    compiled_checksum CHAR(64),
                    compiled_size BIGINT);
                CREATE TABLE compiled_reference(
                    compile_path VARCHAR(255),
                    context_hash BIGINT,
                    source_hash BIGINT,
                    compiled_path VARCHAR(255),
                    compiled_reference VARCHAR(255));
                CREATE TABLE linked_output(
                    id VARCHAR(255),
                    context_hash BIGINT,
                    source_hash BIGINT,
                    checksum CHAR(64),
                    size BIGINT);";

            connection
                .execute(statement)
                .await
                .map_err(Error::Database)?;

            connection
        };
        Ok(Self {
            content: output_content,
            file: output_file,
            database,
        })
    }

    async fn load(path: impl AsRef<Path>, version: &str) -> Result<Self, Error> {
        let output_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())
            .map_err(|_e| Error::NotFound)?;

        let output_content: OutputContent =
            serde_json::from_reader(&output_file).map_err(|e| Error::Io(e.into()))?;

        let database = {
            let db_uri = Self::database_uri(path.as_ref(), version);
            sqlx::any::AnyPoolOptions::new()
                .connect(&db_uri)
                .await
                .map_err(Error::Database)?
        };

        Ok(Self {
            content: output_content,
            file: output_file,
            database,
        })
    }

    pub(crate) async fn open(output_index: &Path, version: &str) -> Result<Self, Error> {
        if !output_index.exists() {
            return Err(Error::NotFound);
        }

        let output_index = Self::load(output_index, version).await?;

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

    pub(crate) async fn insert_compiled(
        &mut self,
        compile_path: &ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
        compiled_resources: &[CompiledResource],
        compiled_references: &[(ResourcePathId, ResourcePathId)],
    ) -> Result<(), Error> {
        // For now we assume there is not concurrent compilation
        // so there is no way to compile the same resources twice.
        // Once we support it we will have to make sure the result of the compilation
        // is exactly the same for all compiled_assets.
        assert!(self
            .find_compiled(compile_path, context_hash, source_hash)
            .await
            .is_none());

        // NOTE: all inserts could be done in one statement instead of for loops
        // but sqlx API doesn't support inserting multiple values in one query
        // at the moment.
        {
            for resource in compiled_resources {
                let query =
                    sqlx::query("INSERT OR REPLACE into compiled_output VALUES(?, ?, ?, ?, ?, ?);")
                        .bind(compile_path.to_string())
                        .bind(context_hash.into_i64())
                        .bind(source_hash.into_i64())
                        .bind(resource.path.to_string())
                        .bind(resource.checksum.to_string())
                        .bind(resource.size as i64);

                self.database
                    .execute(query)
                    .await
                    .map_err(Error::Database)?;
            }

            for (source, dest) in compiled_references {
                let query =
                    sqlx::query("INSERT OR REPLACE into compiled_reference VALUES(?, ?, ?, ?, ?);")
                        .bind(compile_path.to_string())
                        .bind(context_hash.into_i64())
                        .bind(source_hash.into_i64())
                        .bind(source.to_string())
                        .bind(dest.to_string());

                self.database
                    .execute(query)
                    .await
                    .map_err(Error::Database)?;
            }
        }

        let mut compiled_assets_desc: Vec<_> = compiled_resources
            .iter()
            .map(|asset| CompiledResourceInfo {
                compile_path: compile_path.clone(),
                context_hash,
                source_hash,
                compiled_path: asset.path.clone(),
                compiled_checksum: asset.checksum,
                compiled_size: asset.size,
            })
            .collect();

        let mut compiled_references_desc: Vec<_> = compiled_references
            .iter()
            .map(
                |(compiled_guid, compiled_reference)| CompiledResourceReference {
                    compile_path: compile_path.clone(),
                    context_hash,
                    source_hash,
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

        Ok(())
    }

    pub(crate) async fn find_compiled(
        &self,
        compile_path: &ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
    ) -> Option<(Vec<CompiledResourceInfo>, Vec<CompiledResourceReference>)> {
        let (db_compiled, db_references) = {
            let statement = sqlx::query_as(
                "SELECT compiled_path, compiled_checksum, compiled_size 
            FROM compiled_output
            WHERE compile_path = ? AND context_hash = ? AND source_hash = ?",
            )
            .bind(compile_path.to_string())
            .bind(context_hash.into_i64())
            .bind(source_hash.into_i64());

            let result: Vec<(String, String, i64)> =
                statement.fetch_all(&self.database).await.unwrap();
            let compiled = result
                .into_iter()
                .map(|(id, checksum, size)| CompiledResourceInfo {
                    compile_path: compile_path.clone(),
                    context_hash,
                    source_hash,
                    compiled_path: ResourcePathId::from_str(&id).unwrap(),
                    compiled_checksum: Checksum::from_str(&checksum).unwrap(),
                    compiled_size: size as usize,
                })
                .collect::<Vec<_>>();

            let references = if !compiled.is_empty() {
                let statement = sqlx::query_as(
                    "SELECT compiled_path, compiled_reference
                    FROM compiled_reference
                    WHERE compile_path = ? AND context_hash = ? AND source_hash = ?",
                )
                .bind(compile_path.to_string())
                .bind(context_hash.into_i64())
                .bind(source_hash.into_i64());

                let result: Vec<(String, String)> =
                    statement.fetch_all(&self.database).await.unwrap();

                result
                    .into_iter()
                    .map(
                        |(compiled_path, compiled_reference)| CompiledResourceReference {
                            compile_path: compile_path.clone(),
                            context_hash,
                            source_hash,
                            compiled_path: ResourcePathId::from_str(&compiled_path).unwrap(),
                            compiled_reference: ResourcePathId::from_str(&compiled_reference)
                                .unwrap(),
                        },
                    )
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            (compiled, references)
        };

        let asset_objects: Vec<CompiledResourceInfo> = self
            .content
            .compiled_resources
            .iter()
            .filter(|asset| {
                &asset.compile_path == compile_path
                    && asset.context_hash == context_hash
                    && asset.source_hash == source_hash
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
                        && reference.context_hash == context_hash
                        && reference.source_hash == source_hash
                })
                .cloned()
                .collect();

            if asset_objects.len() != db_compiled.len()
                || !asset_objects.iter().all(|e| {
                    let found = db_compiled.contains(e);
                    if !found {
                        println!("not found {:?}", e);
                    }
                    found
                })
            {
                println!("obj: {:?}", asset_objects);
                println!("dbs: {:?}", db_compiled);
                panic!();
            }
            assert_eq!(asset_references, db_references);

            Some((asset_objects, asset_references))
        }
    }

    pub(crate) async fn find_linked(
        &self,
        id: ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
    ) -> Result<Option<(Checksum, usize)>, Error> {
        let db_output = {
            let statement = sqlx::query_as(
                "SELECT checksum, size
                    FROM linked_output
                    WHERE id = ? AND context_hash = ? AND source_hash = ?",
            )
            .bind(id.to_string())
            .bind(context_hash.into_i64())
            .bind(source_hash.into_i64());

            let result: Option<(String, i64)> = statement
                .fetch_optional(&self.database)
                .await
                .map_err(Error::Database)?;

            result.map(|(checksum, size)| (Checksum::from_str(&checksum).unwrap(), size as usize))
        };

        let output = self
            .content
            .linked_resources
            .get(&(id, context_hash, source_hash))
            .copied();
        assert_eq!(output, db_output);

        Ok(output)
    }

    pub(crate) async fn insert_linked(
        &mut self,
        id: ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
        checksum: Checksum,
        size: usize,
    ) -> Result<(), Error> {
        {
            let query = sqlx::query("INSERT OR REPLACE into linked_output VALUES(?, ?, ?, ?, ?);")
                .bind(id.to_string())
                .bind(context_hash.into_i64())
                .bind(source_hash.into_i64())
                .bind(checksum.to_string())
                .bind(size as i64);

            self.database
                .execute(query)
                .await
                .map_err(Error::Database)?;
        }
        self.content
            .linked_resources
            .insert((id, context_hash, source_hash), (checksum, size));

        Ok(())
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

    use lgn_content_store::Checksum;
    use lgn_data_compiler::CompiledResource;
    use lgn_data_offline::ResourcePathId;
    use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};
    use text_resource::TextResource;

    use crate::output_index::{AssetHash, OutputIndex};

    #[tokio::test]
    async fn version_check() {
        let work_dir = tempfile::tempdir().unwrap();

        let buildindex_dir = work_dir.path();
        {
            let _output_index =
                OutputIndex::create_new(&OutputIndex::output_index_file(&buildindex_dir), "0.0.1")
                    .await
                    .unwrap();
        }
        assert!(
            OutputIndex::open(&OutputIndex::output_index_file(&buildindex_dir), "0.0.2")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn create_open() {
        let work_dir = tempfile::tempdir().unwrap();
        let index_path = work_dir.path();
        let index_file = OutputIndex::output_index_file(&index_path);
        {
            let _index = OutputIndex::create_new(&index_file, "0.0.1").await.unwrap();
        }

        let _opened = OutputIndex::open(&index_file, "0.0.1").await.unwrap();
    }

    #[tokio::test]
    async fn outputs() {
        let work_dir = tempfile::tempdir().unwrap();
        let index_path = work_dir.path();
        let index_file = OutputIndex::output_index_file(&index_path);
        let mut index = OutputIndex::create_new(&index_file, "0.0.1").await.unwrap();

        // no dependencies and no references.
        let compile_path = ResourcePathId::from(ResourceTypeAndId {
            kind: TextResource::TYPE,
            id: ResourceId::new(),
        });
        let reference = ResourcePathId::from(ResourceTypeAndId {
            kind: TextResource::TYPE,
            id: ResourceId::new(),
        });
        let context_hash = AssetHash::from(1);
        let source_hash = AssetHash::from(4);
        let in_resources = vec![CompiledResource {
            path: compile_path.clone(),
            checksum: Checksum::from([7u8; 32]),
            size: 9,
        }];
        let references = vec![(compile_path.clone(), reference)];
        index
            .insert_compiled(
                &compile_path,
                context_hash,
                source_hash,
                &in_resources,
                &references,
            )
            .await
            .unwrap();

        let (out_resources, out_references) = index
            .find_compiled(&compile_path, context_hash, source_hash)
            .await
            .unwrap();

        assert_eq!(in_resources.len(), out_resources.len());
        assert_eq!(references.len(), out_references.len());
        for (i, _) in out_resources.iter().enumerate() {
            assert_eq!(compile_path, out_resources[i].compile_path);
            assert_eq!(source_hash, out_resources[i].source_hash);
            assert_eq!(context_hash, out_resources[i].context_hash);
            assert_eq!(in_resources[i].path, out_resources[i].compiled_path);
            assert_eq!(in_resources[i].checksum, out_resources[i].compiled_checksum);
            assert_eq!(in_resources[i].size, out_resources[i].compiled_size);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AssetHash(u64);

impl AssetHash {
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn into_i64(self) -> i64 {
        self.0 as i64
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
