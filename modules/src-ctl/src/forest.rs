use crate::{sql::execute_sql, *};
use futures::executor::block_on;
use sqlx::Row;

// a Forest struct could eventually contain a MRU cache of recently fetched trees

pub fn init_forest_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql =
        "CREATE TABLE tree_nodes (name VARCHAR(255), hash CHAR(64), parent_tree_hash CHAR(64), node_type INTEGER);
         CREATE INDEX tree on tree_nodes(parent_tree_hash);";

    if let Err(e) = execute_sql(sql_connection, sql) {
        return Err(format!("Error creating forest: {}", e));
    }
    Ok(())
}

pub fn read_tree(connection: &mut RepositoryConnection, hash: &str) -> Result<Tree, String> {
    let sql_connection = connection.sql();
    let mut directory_nodes: Vec<TreeNode> = Vec::new();
    let mut file_nodes: Vec<TreeNode> = Vec::new();

    match block_on(
        sqlx::query(
            "SELECT name, hash, node_type
             FROM tree_nodes
             WHERE parent_tree_hash = ?
             ORDER BY name;",
        )
        .bind(hash)
        .fetch_all(&mut *sql_connection),
    ) {
        Ok(rows) => {
            for r in rows {
                let name: String = r.get("name");
                let node_hash: String = r.get("hash");
                let node_type: i64 = r.get("node_type");
                let node = TreeNode::new(name, node_hash);
                if node_type == TreeNodeType::Directory as i64 {
                    directory_nodes.push(node);
                } else if node_type == TreeNodeType::File as i64 {
                    file_nodes.push(node);
                }
            }
        }
        Err(e) => {
            return Err(format!("Error fetching tree nodes for {}: {}", hash, e));
        }
    }

    Ok(Tree {
        directory_nodes,
        file_nodes,
    })
}

pub fn save_tree(
    connection: &mut RepositoryConnection,
    tree: &Tree,
    hash: &str,
) -> Result<(), String> {
    let tree_in_db = read_tree(connection, hash)?;
    if !tree.is_empty() && !tree_in_db.is_empty() {
        return Ok(());
    }

    let sql_connection = connection.sql();

    for file_node in &tree.file_nodes {
        if let Err(e) = block_on(
            sqlx::query("INSERT INTO tree_nodes VALUES(?, ?, ?, ?);")
                .bind(file_node.name.clone())
                .bind(file_node.hash.clone())
                .bind(hash)
                .bind(TreeNodeType::File as i64)
                .execute(&mut *sql_connection),
        ) {
            return Err(format!("Error inserting into tree_nodes: {}", e));
        }
    }

    for dir_node in &tree.directory_nodes {
        if let Err(e) = block_on(
            sqlx::query("INSERT INTO tree_nodes VALUES(?, ?, ?, ?);")
                .bind(dir_node.name.clone())
                .bind(dir_node.hash.clone())
                .bind(hash)
                .bind(TreeNodeType::Directory as i64)
                .execute(&mut *sql_connection),
        ) {
            return Err(format!("Error inserting into tree_nodes: {}", e));
        }
    }

    Ok(())
}
