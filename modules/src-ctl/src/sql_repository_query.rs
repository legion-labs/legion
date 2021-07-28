use crate::{sql::connect, *};
use async_trait::async_trait;

// access to repository metadata inside a mysql or sqlite database
pub struct SqlRepositoryQuery {
    sql_connection: sqlx::AnyConnection,
}

impl SqlRepositoryQuery {
    pub fn new(db_uri: &str) -> Result<SqlRepositoryQuery, String> {
        Ok(Self {
            sql_connection: connect(db_uri)?,
        })
    }
}

#[async_trait]
impl RepositoryQuery for SqlRepositoryQuery {
    fn sql(&mut self) -> &mut sqlx::AnyConnection {
        &mut self.sql_connection
    }
}
