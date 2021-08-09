use crate::sql::execute_sql;

// a Forest struct could eventually contain a MRU cache of recently fetched trees

pub async fn init_forest_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql =
        "CREATE TABLE tree_nodes (name VARCHAR(255), hash CHAR(64), parent_tree_hash CHAR(64), node_type INTEGER);
         CREATE INDEX tree on tree_nodes(parent_tree_hash);";

    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating forest: {}", e));
    }
    Ok(())
}
