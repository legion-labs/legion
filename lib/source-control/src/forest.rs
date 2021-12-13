use anyhow::{Context, Result};

use crate::sql::execute_sql;

// a Forest struct could eventually contain a MRU cache of recently fetched trees

pub async fn init_forest_database(sql_connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql =
        "CREATE TABLE tree_nodes (name VARCHAR(255), hash CHAR(64), parent_tree_hash CHAR(64), node_type INTEGER);
         CREATE INDEX tree on tree_nodes(parent_tree_hash);";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating forest")
}
