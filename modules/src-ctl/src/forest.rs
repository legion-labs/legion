use crate::*;
use std::path::Path;

// a Forest struct could eventually contain a MRU cache of recently fetched trees

fn old_save_tree(repo: &Path, tree: &Tree, hash: &str) -> Result<(), String> {
    let file_path = repo.join("trees").join(String::from(hash) + ".json");
    match serde_json::to_string(&tree) {
        Ok(json) => {
            write_file(&file_path, json.as_bytes())?;
        }
        Err(e) => {
            return Err(format!("Error formatting tree {:?}: {}", tree, e));
        }
    }
    Ok(())
}

pub fn init_forest_database(connection: &Connection) -> Result<(), String> {
    let sql_connection = connection.sql_connection();
    if let Err(e) = sql_connection.execute(
        "CREATE TABLE tree_nodes (name TEXT, hash TEXT, parent_tree_hash TEXT, node_type INTEGER);
         CREATE INDEX tree on tree_nodes(parent_tree_hash);",
    ) {
        return Err(format!("Error creating forest: {}", e));
    }
    Ok(())
}

pub fn read_tree(connection: &Connection, hash: &str) -> Result<Tree, String> {
    let repo = connection.repository();
    let file_path = repo.join(format!("trees/{}.json", hash));
    let parsed: serde_json::Result<Tree> = serde_json::from_str(&read_text_file(&file_path)?);
    match parsed {
        Ok(tree) => Ok(tree),
        Err(e) => Err(format!("Error reading tree {}: {}", hash, e)),
    }
}

pub fn save_tree(connection: &Connection, tree: &Tree, hash: &str) -> Result<(), String> {
    old_save_tree(connection.repository(), tree, hash)?;

    let sql_connection = connection.sql_connection();

    let mut statement = match sql_connection
        .prepare("INSERT INTO tree_nodes VALUES(:name, :hash, :parent_tree_hash, :node_type);")
    {
        Err(e) => {
            return Err(format!("Error preparing insert: {}", e));
        }
        Ok(statement) => statement,
    };

    execute_sql(sql_connection, "BEGIN TRANSACTION;")?;

    statement.bind_by_name(":parent_tree_hash", &*hash).unwrap();
    statement
        .bind_by_name(":node_type", TreeNodeType::File as i64)
        .unwrap();
    let mut cursor = statement.into_cursor();
    for file_node in &tree.file_nodes {
        cursor
            .bind_by_name(vec![
                (":name", sqlite::Value::String(file_node.name.clone())),
                (":hash", sqlite::Value::String(file_node.hash.clone())),
            ])
            .unwrap();
        if let Err(e) = cursor.next() {
            return Err(format!("Error inserting tree_node: {}", e));
        }
    }

    cursor
        .bind_by_name(vec![(
            ":node_type",
            sqlite::Value::Integer(TreeNodeType::Directory as i64),
        )])
        .unwrap();
    for dir_node in &tree.directory_nodes {
        cursor
            .bind_by_name(vec![
                (":name", sqlite::Value::String(dir_node.name.clone())),
                (":hash", sqlite::Value::String(dir_node.hash.clone())),
            ])
            .unwrap();
        if let Err(e) = cursor.next() {
            return Err(format!("Error inserting tree_node: {}", e));
        }
    }

    execute_sql(sql_connection, "COMMIT;")?;

    Ok(())
}
