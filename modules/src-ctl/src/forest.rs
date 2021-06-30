use crate::*;

// a Forest struct could eventually contain a MRU cache of recently fetched trees

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
    let sql_connection = connection.sql_connection();
    let sql = format!(
        "SELECT name, hash, node_type 
         FROM tree_nodes 
         WHERE parent_tree_hash = '{}'
         ORDER BY name
         ;",
        hash
    );
    let mut cursor = sql_connection.prepare(sql).unwrap().into_cursor();

    let mut directory_nodes: Vec<TreeNode> = Vec::new();
    let mut file_nodes: Vec<TreeNode> = Vec::new();

    while let Some(row) = cursor.next().unwrap() {
        let name = row[0].as_string().unwrap();
        let node_hash = row[1].as_string().unwrap();
        let node_type = row[2].as_integer().unwrap();
        let node = TreeNode::new(String::from(name), String::from(node_hash));
        if node_type == TreeNodeType::Directory as i64 {
            directory_nodes.push(node);
        } else if node_type == TreeNodeType::File as i64 {
            file_nodes.push(node);
        }
    }

    Ok(Tree {
        directory_nodes,
        file_nodes,
    })
}

pub fn save_tree(connection: &Connection, tree: &Tree, hash: &str) -> Result<(), String> {
    let tree_in_db = read_tree(connection, hash)?;
    if !tree.is_empty() && !tree_in_db.is_empty() {
        return Ok(());
    }

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
