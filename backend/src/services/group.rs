use sqlx::SqlitePool;
use thiserror::Error;

use crate::models::group::{GroupMember, UserGroup};
use crate::models::user::User;

/// Group service errors
#[derive(Debug, Error)]
pub enum GroupError {
    #[error("Group not found")]
    NotFound,

    #[error("Group name already exists")]
    DuplicateName,

    #[error("Cannot delete system group")]
    CannotDeleteSystemGroup,

    #[error("User is not a member of this group")]
    NotAMember,

    #[error("User is already a member of this group")]
    AlreadyMember,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Create a new group
pub async fn create_group(
    db: &SqlitePool,
    name: &str,
    description: Option<String>,
) -> Result<UserGroup, GroupError> {
    // Check if group name already exists
    let existing = get_group_by_name(db, name).await?;
    if existing.is_some() {
        return Err(GroupError::DuplicateName);
    }

    let group = UserGroup::new(name.to_string(), description, false);

    sqlx::query(
        r#"
        INSERT INTO user_groups (id, name, description, is_system, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&group.id)
    .bind(&group.name)
    .bind(&group.description)
    .bind(group.is_system)
    .bind(&group.created_at)
    .bind(&group.updated_at)
    .execute(db)
    .await?;

    Ok(group)
}

/// Get a group by ID
pub async fn get_group_by_id(db: &SqlitePool, id: &str) -> Result<Option<UserGroup>, GroupError> {
    let group = sqlx::query_as::<_, UserGroup>("SELECT * FROM user_groups WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?;

    Ok(group)
}

/// Get a group by name
pub async fn get_group_by_name(
    db: &SqlitePool,
    name: &str,
) -> Result<Option<UserGroup>, GroupError> {
    let group = sqlx::query_as::<_, UserGroup>("SELECT * FROM user_groups WHERE name = ?")
        .bind(name)
        .fetch_optional(db)
        .await?;

    Ok(group)
}

/// List all groups
pub async fn list_groups(db: &SqlitePool) -> Result<Vec<UserGroup>, GroupError> {
    let groups =
        sqlx::query_as::<_, UserGroup>("SELECT * FROM user_groups ORDER BY is_system DESC, name")
            .fetch_all(db)
            .await?;

    Ok(groups)
}

/// Update a group
pub async fn update_group(
    db: &SqlitePool,
    id: &str,
    name: Option<String>,
    description: Option<Option<String>>,
) -> Result<UserGroup, GroupError> {
    let existing = get_group_by_id(db, id).await?.ok_or(GroupError::NotFound)?;

    let now = chrono::Utc::now().to_rfc3339();
    let new_name = name.unwrap_or(existing.name.clone());
    let new_description = description.unwrap_or(existing.description.clone());

    // Check for duplicate name if changing
    if new_name != existing.name {
        if get_group_by_name(db, &new_name).await?.is_some() {
            return Err(GroupError::DuplicateName);
        }
    }

    sqlx::query(
        r#"
        UPDATE user_groups
        SET name = ?, description = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&new_name)
    .bind(&new_description)
    .bind(&now)
    .bind(id)
    .execute(db)
    .await?;

    get_group_by_id(db, id).await?.ok_or(GroupError::NotFound)
}

/// Delete a group
pub async fn delete_group(db: &SqlitePool, id: &str) -> Result<(), GroupError> {
    let group = get_group_by_id(db, id).await?.ok_or(GroupError::NotFound)?;

    if group.is_system {
        return Err(GroupError::CannotDeleteSystemGroup);
    }

    sqlx::query("DELETE FROM user_groups WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;

    Ok(())
}

/// Add a user to a group
pub async fn add_member(
    db: &SqlitePool,
    group_id: &str,
    user_id: &str,
) -> Result<(), GroupError> {
    // Check if group exists
    let _ = get_group_by_id(db, group_id)
        .await?
        .ok_or(GroupError::NotFound)?;

    // Check if already a member
    let existing = sqlx::query_as::<_, GroupMember>(
        "SELECT * FROM user_group_members WHERE group_id = ? AND user_id = ?",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    if existing.is_some() {
        return Err(GroupError::AlreadyMember);
    }

    let member = GroupMember::new(user_id.to_string(), group_id.to_string());

    sqlx::query(
        r#"
        INSERT INTO user_group_members (id, user_id, group_id, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&member.id)
    .bind(&member.user_id)
    .bind(&member.group_id)
    .bind(&member.created_at)
    .execute(db)
    .await?;

    Ok(())
}

/// Remove a user from a group
pub async fn remove_member(
    db: &SqlitePool,
    group_id: &str,
    user_id: &str,
) -> Result<(), GroupError> {
    let result = sqlx::query("DELETE FROM user_group_members WHERE group_id = ? AND user_id = ?")
        .bind(group_id)
        .bind(user_id)
        .execute(db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(GroupError::NotAMember);
    }

    Ok(())
}

/// Get all groups a user belongs to
pub async fn get_user_groups(db: &SqlitePool, user_id: &str) -> Result<Vec<UserGroup>, GroupError> {
    let groups = sqlx::query_as::<_, UserGroup>(
        r#"
        SELECT g.* FROM user_groups g
        INNER JOIN user_group_members m ON g.id = m.group_id
        WHERE m.user_id = ?
        ORDER BY g.name
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(groups)
}

/// Get all members of a group
pub async fn get_group_members(db: &SqlitePool, group_id: &str) -> Result<Vec<User>, GroupError> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT u.* FROM users u
        INNER JOIN user_group_members m ON u.id = m.user_id
        WHERE m.group_id = ?
        ORDER BY u.username
        "#,
    )
    .bind(group_id)
    .fetch_all(db)
    .await?;

    Ok(users)
}

/// Count members in a group
pub async fn count_group_members(db: &SqlitePool, group_id: &str) -> Result<i64, GroupError> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_group_members WHERE group_id = ?")
            .bind(group_id)
            .fetch_one(db)
            .await?;

    Ok(count.0)
}
