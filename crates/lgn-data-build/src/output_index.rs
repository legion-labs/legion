use std::str::FromStr;

use lgn_content_store::Identifier;
use lgn_data_compiler::CompiledResource;
use lgn_data_runtime::ResourcePathId;
use lgn_data_runtime::ResourceTypeAndId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::Connection;
use sqlx::SqliteConnection;
use sqlx::{migrate::MigrateDatabase, Executor, Row};

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
    pub(crate) compiled_content_id: Identifier,
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
    content_id: Identifier,
}

#[derive(Debug)]
pub(crate) struct OutputIndex {
    uri: String,
}

impl OutputIndex {
    pub(crate) async fn create_new(db_uri: String) -> Result<Self, Error> {
        sqlx::Any::create_database(&db_uri)
            .await
            .map_err(Error::Database)?;
        let connection = sqlx::any::AnyPoolOptions::new()
            .max_connections(10)
            .connect(&db_uri)
            .await
            .map_err(Error::Database)?;

        let statement = "
                CREATE TABLE compiled_output(
                    compile_path VARCHAR(255),
                    context_hash BIGINT,
                    source_hash BIGINT,
                    compiled_path VARCHAR(255),
                    compiled_checksum VARCHAR(255));
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
                    checksum VARCHAR(255));
                CREATE TABLE pathid_mapping(
                    resource_id VARCHAR(255), 
                    resource_path_id VARCHAR(255));";

        connection
            .execute(statement)
            .await
            .map_err(Error::Database)?;

        Ok(Self { uri: db_uri })
    }

    async fn load(db_uri: String) -> Result<Self, Error> {
        if sqlx::Any::database_exists(&db_uri)
            .await
            .map_err(Error::Database)?
        {
            Ok(Self { uri: db_uri })
        } else {
            Err(Error::NotFound(db_uri))
        }
    }

    pub(crate) async fn open(db_uri: String) -> Result<Self, Error> {
        if !sqlx::Any::database_exists(&db_uri)
            .await
            .map_err(Error::Database)?
        {
            return Err(Error::NotFound(db_uri));
        }

        let output_index = Self::load(db_uri).await?;

        Ok(output_index)
    }

    pub(crate) async fn insert_compiled(
        &self,
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

        let mut conn = SqliteConnection::connect(&self.uri)
            .await
            .map_err(Error::Database)?;
        {
            for resource in compiled_resources {
                let query = sqlx::query("INSERT into compiled_output VALUES(?, ?, ?, ?, ?);")
                    .bind(compile_path.to_string())
                    .bind(context_hash.into_i64())
                    .bind(source_hash.into_i64())
                    .bind(resource.path.to_string())
                    .bind(resource.content_id.to_string());

                conn.execute(query).await.map_err(Error::Database)?;
            }

            for (source, dest) in compiled_references {
                let query = sqlx::query("INSERT into compiled_reference VALUES(?, ?, ?, ?, ?);")
                    .bind(compile_path.to_string())
                    .bind(context_hash.into_i64())
                    .bind(source_hash.into_i64())
                    .bind(source.to_string())
                    .bind(dest.to_string());

                conn.execute(query).await.map_err(Error::Database)?;
            }
        }

        Ok(())
    }

    pub(crate) async fn find_compiled(
        &self,
        compile_path: &ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
    ) -> Option<(Vec<CompiledResourceInfo>, Vec<CompiledResourceReference>)> {
        let statement = sqlx::query_as(
            "SELECT compiled_path, compiled_checksum 
            FROM compiled_output
            WHERE compile_path = ? AND context_hash = ? AND source_hash = ?",
        )
        .bind(compile_path.to_string())
        .bind(context_hash.into_i64())
        .bind(source_hash.into_i64());

        let mut conn = SqliteConnection::connect(&self.uri).await.unwrap();

        let result: Vec<(String, String)> = statement.fetch_all(&mut conn).await.unwrap();
        let compiled = result
            .into_iter()
            .map(|(id, checksum)| CompiledResourceInfo {
                compile_path: compile_path.clone(),
                context_hash,
                source_hash,
                compiled_path: ResourcePathId::from_str(&id).unwrap(),
                compiled_content_id: Identifier::from_str(&checksum).unwrap(),
            })
            .collect::<Vec<_>>();

        if compiled.is_empty() {
            return None;
        }

        let references = {
            let statement = sqlx::query_as(
                "SELECT compiled_path, compiled_reference
                    FROM compiled_reference
                    WHERE compile_path = ? AND context_hash = ? AND source_hash = ?",
            )
            .bind(compile_path.to_string())
            .bind(context_hash.into_i64())
            .bind(source_hash.into_i64());

            let result: Vec<(String, String)> = statement.fetch_all(&mut conn).await.unwrap();

            result
                .into_iter()
                .map(
                    |(compiled_path, compiled_reference)| CompiledResourceReference {
                        compile_path: compile_path.clone(),
                        context_hash,
                        source_hash,
                        compiled_path: ResourcePathId::from_str(&compiled_path).unwrap(),
                        compiled_reference: ResourcePathId::from_str(&compiled_reference).unwrap(),
                    },
                )
                .collect::<Vec<_>>()
        };
        Some((compiled, references))
    }

    pub(crate) async fn find_linked(
        &self,
        id: ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
    ) -> Result<Option<Identifier>, Error> {
        let output = {
            let mut conn = SqliteConnection::connect(&self.uri)
                .await
                .map_err(Error::Database)?;
            let statement = sqlx::query_as(
                "SELECT checksum
                    FROM linked_output
                    WHERE id = ? AND context_hash = ? AND source_hash = ?",
            )
            .bind(id.to_string())
            .bind(context_hash.into_i64())
            .bind(source_hash.into_i64());

            let result: Option<(String,)> = statement
                .fetch_optional(&mut conn)
                .await
                .map_err(Error::Database)?;

            result.map(|(checksum,)| Identifier::from_str(&checksum).unwrap())
        };

        Ok(output)
    }

    pub(crate) async fn insert_linked(
        &self,
        id: ResourcePathId,
        context_hash: AssetHash,
        source_hash: AssetHash,
        content_id: Identifier,
    ) -> Result<(), Error> {
        let mut conn = SqliteConnection::connect(&self.uri)
            .await
            .map_err(Error::Database)?;
        let query = sqlx::query("INSERT into linked_output VALUES(?, ?, ?, ?);")
            .bind(id.to_string())
            .bind(context_hash.into_i64())
            .bind(source_hash.into_i64())
            .bind(content_id.to_string());

        conn.execute(query).await.map_err(Error::Database)?;

        Ok(())
    }

    pub async fn record_pathid(&mut self, id: &ResourcePathId) -> Result<(), Error> {
        let query = sqlx::query("INSERT into pathid_mapping VALUES(?, ?);")
            .bind(id.resource_id().to_string())
            .bind(id.to_string());

        let mut conn = SqliteConnection::connect(&self.uri)
            .await
            .map_err(Error::Database)?;

        conn.execute(query).await.map_err(Error::Database)?;

        Ok(())
    }

    pub async fn lookup_pathid(
        &self,
        id: ResourceTypeAndId,
    ) -> Result<Option<ResourcePathId>, Error> {
        let mut conn = SqliteConnection::connect(&self.uri)
            .await
            .map_err(Error::Database)?;
        let output = {
            let statement = sqlx::query(
                "SELECT resource_path_id
                    FROM pathid_mapping
                    WHERE resource_id = ?",
            )
            .bind(id.to_string());

            let result = statement
                .fetch_optional(&mut conn)
                .await
                .map_err(Error::Database)?;

            if let Some(id) = result {
                let id: String = id.get("resource_path_id");
                Some(ResourcePathId::from_str(&id).unwrap())
            } else {
                None
            }
        };

        Ok(output)
    }
}

#[cfg(test)]
mod tests {

    use std::{path::Path, str::FromStr, thread, time::Duration};

    use futures::future::join_all;
    use lgn_content_store::Identifier;
    use lgn_data_compiler::CompiledResource;
    use lgn_data_runtime::{ResourceDescriptor, ResourceId, ResourcePathId, ResourceTypeAndId};
    use sqlx::{migrate::MigrateDatabase, Executor};
    use text_resource::TextResource;

    use crate::output_index::{AssetHash, OutputIndex};

    #[tokio::test]
    async fn sqlx_timeout() {
        let work_dir = tempfile::tempdir().unwrap();
        let index_path = work_dir.path();
        let index_db = test_database_uri(&index_path, "0.0.1");

        sqlx::Any::create_database(&index_db).await.unwrap();

        let connection = sqlx::any::AnyPoolOptions::new()
            .max_connections(1)
            .connect_timeout(Duration::from_millis(300))
            .connect(&index_db)
            .await
            .unwrap();

        connection
            .execute("CREATE TABLE some_table(some_number BIGINT);")
            .await
            .unwrap();

        let mut vec_future = Vec::new();

        for _ in 1..3 {
            vec_future.push(async {
                connection
                    .execute(sqlx::query("SELECT * FROM some_table"))
                    .await
                    .unwrap();
                thread::sleep(Duration::from_secs(1));
                connection
                    .execute(sqlx::query("SELECT * FROM some_table"))
                    .await
                    .unwrap();
            });
        }

        // sqlx will return PoolTimedOut here
        let _results = join_all(vec_future).await;
    }

    fn test_database_uri(buildindex_dir: impl AsRef<Path>, version: &str) -> String {
        let db_path = buildindex_dir
            .as_ref()
            .join(format!("output-{}.db3", version));
        format!("sqlite://{}", db_path.to_str().unwrap().replace('\\', "/"))
    }

    #[tokio::test]
    async fn version_check() {
        let work_dir = tempfile::tempdir().unwrap();

        let buildindex_dir = work_dir.path();
        {
            let _output_index =
                OutputIndex::create_new(test_database_uri(&buildindex_dir, "0.0.1"))
                    .await
                    .unwrap();
        }
        assert!(
            OutputIndex::open(test_database_uri(&buildindex_dir, "0.0.2"))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn create_open() {
        let work_dir = tempfile::tempdir().unwrap();
        let index_path = work_dir.path();
        let index_db = test_database_uri(&index_path, "0.0.1");
        {
            let _index = OutputIndex::create_new(index_db.clone()).await.unwrap();
        }

        let _opened = OutputIndex::open(index_db).await.unwrap();
    }

    #[tokio::test]
    async fn outputs() {
        let work_dir = tempfile::tempdir().unwrap();
        let index_path = work_dir.path();
        let index_db = test_database_uri(&index_path, "0.0.1");
        let index = OutputIndex::create_new(index_db).await.unwrap();

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
            content_id: Identifier::from_str("AAAA").unwrap(),
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
            assert_eq!(
                in_resources[i].content_id,
                out_resources[i].compiled_content_id
            );
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
