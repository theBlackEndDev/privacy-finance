use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::category::{Category, CategoryGroup, CreateCategoryInput, UpdateCategoryInput};

#[tauri::command]
pub fn get_categories(state: State<DbState>) -> AppResult<Vec<Category>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.color, c.icon, c.type, c.sort_order, c.is_archived,
                c.group_id, cg.name, c.created_at, c.updated_at
         FROM categories c
         LEFT JOIN category_groups cg ON c.group_id = cg.id
         ORDER BY c.type ASC, c.sort_order ASC, c.name ASC"
    )?;
    let cats = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            icon: row.get(3)?,
            category_type: row.get(4)?,
            sort_order: row.get(5)?,
            is_archived: row.get::<_, i64>(6)? != 0,
            group_id: row.get(7)?,
            group_name: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(cats)
}

#[tauri::command]
pub fn create_category(input: CreateCategoryInput, state: State<DbState>) -> AppResult<Category> {
    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let color = input.color.unwrap_or_else(|| "#6366f1".to_string());
    let icon = input.icon.unwrap_or_else(|| "folder".to_string());
    let sort = input.sort_order.unwrap_or(0);

    conn.execute(
        "INSERT INTO categories (id, name, color, icon, type, sort_order, is_archived, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        rusqlite::params![id, input.name, color, icon, input.category_type, sort, now, now],
    )?;

    Ok(Category {
        id,
        name: input.name,
        color,
        icon,
        category_type: input.category_type,
        sort_order: sort,
        is_archived: false,
        group_id: None,
        group_name: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_category(id: String, input: UpdateCategoryInput, state: State<DbState>) -> AppResult<Category> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    let current = conn.query_row(
        "SELECT name, color, icon, type, sort_order, is_archived, group_id FROM categories WHERE id = ?1",
        [&id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, Option<String>>(6)?,
        )),
    ).map_err(|_| AppError::NotFound(format!("category {id}")))?;

    let name = input.name.unwrap_or(current.0);
    let color = input.color.unwrap_or(current.1);
    let icon = input.icon.unwrap_or(current.2);
    let category_type = input.category_type.unwrap_or(current.3);
    let sort = input.sort_order.unwrap_or(current.4);
    let archived = input.is_archived.map(|b| b as i64).unwrap_or(current.5);
    let group_id = match input.group_id {
        Some(gid) => gid,     // explicit set (Some(id)) or clear (None)
        None => current.6,    // not provided — keep current
    };

    conn.execute(
        "UPDATE categories SET name=?1, color=?2, icon=?3, type=?4, sort_order=?5, is_archived=?6, group_id=?7, updated_at=?8 WHERE id=?9",
        rusqlite::params![name, color, icon, category_type, sort, archived, group_id, now, id],
    )?;

    drop(conn);

    let read = state.read_conn.lock().unwrap();
    let (created_at, group_name): (String, Option<String>) = read.query_row(
        "SELECT c.created_at, cg.name FROM categories c LEFT JOIN category_groups cg ON c.group_id = cg.id WHERE c.id = ?1",
        [&id], |r| Ok((r.get(0)?, r.get(1)?))
    ).map_err(|_| AppError::NotFound(format!("category {id}")))?;

    Ok(Category {
        id,
        name,
        color,
        icon,
        category_type,
        sort_order: sort,
        is_archived: archived != 0,
        group_id,
        group_name,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_category(id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM categories WHERE id = ?1", [&id], |r| r.get::<_, i64>(0)
    ).map(|c| c > 0).unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound(format!("category {id}")));
    }

    // ON DELETE SET NULL — transactions become uncategorized automatically
    conn.execute("DELETE FROM categories WHERE id = ?1", [&id])?;

    Ok(())
}

// ── Category Groups ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_category_groups(state: State<DbState>) -> AppResult<Vec<CategoryGroup>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, created_at, updated_at FROM category_groups ORDER BY sort_order ASC, name ASC"
    )?;
    let groups = stmt.query_map([], |row| {
        Ok(CategoryGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(groups)
}

#[tauri::command]
pub fn create_category_group(name: String, sort_order: Option<i64>, state: State<DbState>) -> AppResult<CategoryGroup> {
    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let sort = sort_order.unwrap_or(0);

    conn.execute(
        "INSERT INTO category_groups (id, name, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        rusqlite::params![id, name, sort, now],
    )?;

    Ok(CategoryGroup { id, name, sort_order: sort, created_at: now.clone(), updated_at: now })
}
