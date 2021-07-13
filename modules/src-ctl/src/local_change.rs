use crate::*;
use futures::executor::block_on;
use sqlx::Row;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Edit = 1,
    Add = 2,
    Delete = 3,
}

impl ChangeType {
    pub fn from_int(i: i64) -> Result<Self, String> {
        match i {
            1 => Ok(Self::Edit),
            2 => Ok(Self::Add),
            3 => Ok(Self::Delete),
            _ => Err(format!("Invalid change type {}", i)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalChange {
    pub relative_path: String,
    pub change_type: ChangeType,
}

impl LocalChange {
    pub fn new(canonical_relative_path: &str, change_type: ChangeType) -> Self {
        Self {
            relative_path: canonical_relative_path.to_lowercase(),
            change_type,
        }
    }
}

pub fn init_local_changes_database(
    connection: &mut LocalWorkspaceConnection,
) -> Result<(), String> {
    let sql_connection = connection.sql();
    let sql = "CREATE TABLE changes(relative_path TEXT NOT NULL PRIMARY KEY, change_type INTEGER);";
    if let Err(e) = execute_sql(sql_connection, sql) {
        return Err(format!("Error creating change table: {}", e));
    }
    Ok(())
}

pub fn save_local_change(
    connection: &mut LocalWorkspaceConnection,
    change_spec: &LocalChange,
) -> Result<(), String> {
    let sql_connection = connection.sql();
    if let Err(e) = block_on(
        sqlx::query("REPLACE INTO changes VALUES(?, ?);")
            .bind(change_spec.relative_path.clone())
            .bind(change_spec.change_type.clone() as i64)
            .execute(sql_connection),
    ) {
        return Err(format!(
            "Error saving local change to {}: {}",
            change_spec.relative_path, e
        ));
    }
    Ok(())
}

pub fn find_local_change(
    connection: &mut LocalWorkspaceConnection,
    canonical_relative_path: &str,
) -> Result<Option<LocalChange>, String> {
    let path = canonical_relative_path.to_lowercase();
    let sql_connection = connection.sql();

    match block_on(
        sqlx::query(
            "SELECT change_type
             FROM changes
             WHERE relative_path = ?;",
        )
        .bind(path.clone())
        .fetch_optional(&mut *sql_connection),
    ) {
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Error fetching local change: {}", e)),
        Ok(Some(row)) => {
            let change_type_int: i64 = row.get("change_type");
            Ok(Some(LocalChange::new(
                &path,
                ChangeType::from_int(change_type_int).unwrap(),
            )))
        }
    }
}

pub fn read_local_changes(
    connection: &mut LocalWorkspaceConnection,
) -> Result<Vec<LocalChange>, String> {
    let sql_connection = connection.sql();
    match block_on(
        sqlx::query(
            "SELECT relative_path, change_type
             FROM changes",
        )
        .fetch_all(sql_connection),
    ) {
        Ok(rows) => {
            let mut res = Vec::new();
            for row in rows {
                let change_type_int: i64 = row.get("change_type");
                res.push(LocalChange::new(
                    row.get("relative_path"),
                    ChangeType::from_int(change_type_int).unwrap(),
                ));
            }
            Ok(res)
        }
        Err(e) => Err(format!("Error reading local changes: {}", e)),
    }
}

pub fn find_local_changes_command() -> Result<Vec<LocalChange>, String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    read_local_changes(&mut workspace_connection)
}

pub fn clear_local_change(
    connection: &mut LocalWorkspaceConnection,
    change: &LocalChange,
) -> Result<(), String> {
    let sql_connection = connection.sql();
    if let Err(e) = block_on(
        sqlx::query("DELETE from changes where relative_path=?;")
            .bind(change.relative_path.clone())
            .execute(sql_connection),
    ) {
        return Err(format!(
            "Error clearing local change {}: {}",
            change.relative_path, e
        ));
    }
    Ok(())
}

pub fn clear_local_changes(
    connection: &mut LocalWorkspaceConnection,
    local_changes: &[LocalChange],
) {
    for change in local_changes {
        if let Err(e) = clear_local_change(connection, change) {
            println!(
                "Error clearing local change {}: {}",
                change.relative_path, e
            );
        }
    }
}

pub fn track_new_file_command(path_specified: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path_specified);
    if let Err(e) = fs::metadata(&abs_path) {
        return Err(format!(
            "Error reading file metadata {}: {}",
            &abs_path.display(),
            e
        ));
    }
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    //todo: make sure the file does not exist in the current tree hierarchy

    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;
    match find_local_change(&mut workspace_connection, &relative_path) {
        Ok(Some(change)) => {
            return Err(format!(
                "Error: {} already tracked for {:?}",
                change.relative_path, change.change_type
            ));
        }
        Err(e) => {
            return Err(format!("Error searching in local changes: {}", e));
        }
        Ok(None) => { //all is good
        }
    }

    let current_branch = read_current_branch(&workspace_root)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let mut connection = connect_to_server(&workspace_spec)?;

    if let Some(_hash) = find_file_hash_at_commit(
        &mut connection,
        Path::new(&relative_path),
        &current_branch.head,
    )? {
        return Err(String::from("file already exists in tree"));
    }

    assert_not_locked(&workspace_root, &abs_path)?;
    let local_change = LocalChange::new(&relative_path, ChangeType::Add);

    save_local_change(&mut workspace_connection, &local_change)
}

pub fn edit_file_command(path_specified: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path_specified);
    if let Err(e) = fs::metadata(&abs_path) {
        return Err(format!(
            "Error reading file metadata {}: {}",
            &abs_path.display(),
            e
        ));
    }

    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    //todo: make sure file is tracked by finding it in the current tree hierarchy
    assert_not_locked(&workspace_root, &abs_path)?;

    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;
    match find_local_change(&mut workspace_connection, &relative_path) {
        Ok(Some(change)) => {
            return Err(format!(
                "Error: {} already tracked for {:?}",
                change.relative_path, change.change_type
            ));
        }
        Err(e) => {
            return Err(format!("Error searching in local changes: {}", e));
        }
        Ok(None) => { //all is good
        }
    }

    let local_change = LocalChange::new(&relative_path, ChangeType::Edit);
    save_local_change(&mut workspace_connection, &local_change)?;
    make_file_read_only(&abs_path, false)
}
