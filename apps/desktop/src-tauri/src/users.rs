use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct UserListItem {
    pub id: String,
    pub name: String,
    pub status: String,
    pub outlet_id: Option<String>,
    pub roles: Vec<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleItem {
    pub id: String,
    pub name: String,
    pub system_role: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRoleAssignment {
    pub id: String,
    pub role_id: String,
    pub role_name: String,
    pub outlet_id: String,
    pub status: String,
    pub valid_from: String,
    pub valid_until: String,
    pub assigned_by: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserDetail {
    pub id: String,
    pub name: String,
    pub status: String,
    pub outlet_id: Option<String>,
    pub failed_login_attempts: i64,
    pub locked_until: Option<String>,
    pub created_at: String,
    pub roles: Vec<UserRoleAssignment>,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

async fn require_permission(
    pool: &SqlitePool,
    permission_key: &str,
) -> Result<String, String> {
    let user_id = crate::auth::get_current_user(pool).await?;
    let has_perm = crate::auth::has_permission(pool, user_id, permission_key).await?;
    if !has_perm {
        return Err(format!(
            "Akses ditolak: Anda tidak memiliki izin '{}'",
            permission_key
        ));
    }
    Ok(user_id.to_string())
}

// ── Commands ─────────────────────────────────────────────────────────────────

/// Daftar semua user dalam satu outlet.
#[tauri::command]
pub async fn list_users(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<UserListItem>, String> {
    require_permission(pool.inner(), "user.view").await?;
    let outlet_id = crate::auth::get_current_outlet(pool.inner())
        .await
        .map(|id| id.to_string())
        .unwrap_or_default();

    let rows = sqlx::query(
        r#"
        SELECT u.id, u.name, u.status, u.outlet_id, u.created_at
        FROM users u
        WHERE u.outlet_id = ? OR u.outlet_id IS NULL
        ORDER BY u.created_at DESC
        "#,
    )
    .bind(&outlet_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut users = Vec::new();
    for row in rows {
        let uid: String = row.get("id");

        // Load roles for this user
        let role_rows = sqlx::query(
            r#"
            SELECT r.name
            FROM user_outlet_roles uor
            JOIN roles r ON uor.role_id = r.id
            WHERE uor.user_id = ? AND uor.outlet_id = ? AND uor.status = 'ACTIVE'
            AND uor.valid_from <= datetime('now') AND uor.valid_until >= datetime('now')
            "#,
        )
        .bind(&uid)
        .bind(&outlet_id)
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

        let roles: Vec<String> = role_rows.iter().map(|r| r.get("name")).collect();

        users.push(UserListItem {
            id: uid,
            name: row.get("name"),
            status: row.get("status"),
            outlet_id: row.get("outlet_id"),
            roles,
            created_at: row.get("created_at"),
        });
    }

    Ok(users)
}

/// Detail satu user termasuk riwayat role assignment.
#[tauri::command]
pub async fn get_user_detail(
    user_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<UserDetail, String> {
    require_permission(pool.inner(), "user.view").await?;

    let row = sqlx::query(
        "SELECT id, name, status, outlet_id, failed_login_attempts, locked_until, created_at FROM users WHERE id = ?",
    )
    .bind(&user_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .ok_or("Pengguna tidak ditemukan".to_string())?;

    let role_rows = sqlx::query(
        r#"
        SELECT uor.id, uor.role_id, r.name AS role_name, uor.outlet_id,
               uor.status, uor.valid_from, uor.valid_until, uor.assigned_by
        FROM user_outlet_roles uor
        JOIN roles r ON uor.role_id = r.id
        WHERE uor.user_id = ?
        ORDER BY uor.created_at DESC
        "#,
    )
    .bind(&user_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let roles = role_rows
        .iter()
        .map(|r| UserRoleAssignment {
            id: r.get("id"),
            role_id: r.get("role_id"),
            role_name: r.get("role_name"),
            outlet_id: r.get("outlet_id"),
            status: r.get("status"),
            valid_from: r.get("valid_from"),
            valid_until: r.get("valid_until"),
            assigned_by: r.get("assigned_by"),
        })
        .collect();

    Ok(UserDetail {
        id: row.get("id"),
        name: row.get("name"),
        status: row.get("status"),
        outlet_id: row.get("outlet_id"),
        failed_login_attempts: row.get("failed_login_attempts"),
        locked_until: row.get("locked_until"),
        created_at: row.get("created_at"),
        roles,
    })
}

/// Buat user baru dengan PIN awal dan langsung assign role di outlet.
#[tauri::command]
pub async fn create_user(
    name: String,
    initial_pin: String,
    role_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    let actor_id = require_permission(pool.inner(), "user.create").await?;
    let outlet_id = crate::auth::get_current_outlet(pool.inner())
        .await
        .map(|id| id.to_string())?;

    // Validate
    if name.trim().is_empty() {
        return Err("Nama pengguna tidak boleh kosong".to_string());
    }
    if initial_pin.len() < 4 {
        return Err("PIN minimal 4 digit".to_string());
    }

    // Check duplicate name in same outlet
    let existing: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM users WHERE name = ? AND (outlet_id = ? OR outlet_id IS NULL)",
    )
    .bind(&name)
    .bind(&outlet_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;
    if existing > 0 {
        return Err("Nama pengguna sudah digunakan di outlet ini".to_string());
    }

    // Get merchant_id from outlet
    let merchant_id: String = sqlx::query_scalar("SELECT merchant_id FROM outlets WHERE id = ?")
        .bind(&outlet_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|_| "Outlet tidak ditemukan".to_string())?;

    let pin_hash = crate::auth::hash_pin_argon2(&initial_pin)?;
    let new_user_id = Uuid::new_v4().to_string();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO users (id, merchant_id, outlet_id, name, pin_hash_v2, status) VALUES (?, ?, ?, ?, ?, 'active')",
    )
    .bind(&new_user_id)
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(&name)
    .bind(&pin_hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Assign role via user_roles (global, legacy)
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(&new_user_id)
        .bind(&role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Assign role via user_outlet_roles (outlet-scoped RBAC)
    let uor_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO user_outlet_roles (id, user_id, outlet_id, role_id, valid_from, valid_until, status, assigned_by)
        VALUES (?, ?, ?, ?, datetime('now', '-1 day'), datetime('now', '+5 years'), 'ACTIVE', ?)
        "#,
    )
    .bind(&uor_id)
    .bind(&new_user_id)
    .bind(&outlet_id)
    .bind(&role_id)
    .bind(&actor_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    // Audit log
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let _ = crate::audit::log_action(
        &mut *conn,
        merchant_id,
        Some(outlet_id),
        actor_id,
        "user.create",
        "users",
        Some(new_user_id.clone()),
        Some(&format!("Pengguna '{}' dibuat dengan role_id={}", name, role_id)),
    )
    .await;

    Ok(new_user_id)
}

/// Update status user (active / inactive).
#[tauri::command]
pub async fn update_user_status(
    user_id: String,
    new_status: String, // "active" | "inactive"
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let actor_id = require_permission(pool.inner(), "user.deactivate").await?;

    if new_status != "active" && new_status != "inactive" {
        return Err("Status tidak valid. Gunakan 'active' atau 'inactive'".to_string());
    }

    // Prevent self-deactivation
    if user_id == actor_id {
        return Err("Anda tidak dapat mengubah status akun Anda sendiri".to_string());
    }

    let row = sqlx::query("SELECT name, outlet_id FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Pengguna tidak ditemukan".to_string())?;

    let name: String = row.get("name");
    let outlet_id: Option<String> = row.get("outlet_id");
    let merchant_id: String = sqlx::query_scalar(
        "SELECT merchant_id FROM outlets WHERE id = ?",
    )
    .bind(outlet_id.as_deref().unwrap_or(""))
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    sqlx::query("UPDATE users SET status = ? WHERE id = ?")
        .bind(&new_status)
        .bind(&user_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    // Audit
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let _ = crate::audit::log_action(
        &mut *conn,
        merchant_id,
        outlet_id,
        actor_id,
        "user.update_status",
        "users",
        Some(user_id),
        Some(&format!("Status pengguna '{}' diubah menjadi '{}'", name, new_status)),
    )
    .await;

    Ok(())
}

/// Reset PIN user.
#[tauri::command]
pub async fn reset_user_pin(
    user_id: String,
    new_pin: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let actor_id = require_permission(pool.inner(), "user.reset_pin").await?;

    if new_pin.len() < 4 {
        return Err("PIN minimal 4 digit".to_string());
    }

    let row = sqlx::query("SELECT name, outlet_id FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Pengguna tidak ditemukan".to_string())?;

    let name: String = row.get("name");
    let outlet_id: Option<String> = row.get("outlet_id");
    let merchant_id: String = sqlx::query_scalar(
        "SELECT merchant_id FROM outlets WHERE id = ?",
    )
    .bind(outlet_id.as_deref().unwrap_or(""))
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    let new_hash = crate::auth::hash_pin_argon2(&new_pin)?;

    sqlx::query(
        "UPDATE users SET pin_hash_v2 = ?, failed_login_attempts = 0, locked_until = NULL WHERE id = ?",
    )
    .bind(&new_hash)
    .bind(&user_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    // Audit (do NOT log the PIN itself)
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let _ = crate::audit::log_action(
        &mut *conn,
        merchant_id,
        outlet_id,
        actor_id,
        "user.reset_pin",
        "users",
        Some(user_id),
        Some(&format!("PIN pengguna '{}' telah di-reset", name)),
    )
    .await;

    Ok(())
}

/// Edit nama user.
#[tauri::command]
pub async fn update_user_name(
    user_id: String,
    new_name: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let actor_id = require_permission(pool.inner(), "user.edit").await?;

    if new_name.trim().is_empty() {
        return Err("Nama tidak boleh kosong".to_string());
    }

    let row = sqlx::query("SELECT outlet_id FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Pengguna tidak ditemukan".to_string())?;
    let outlet_id: Option<String> = row.get("outlet_id");
    let merchant_id: String = sqlx::query_scalar(
        "SELECT merchant_id FROM outlets WHERE id = ?",
    )
    .bind(outlet_id.as_deref().unwrap_or(""))
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    sqlx::query("UPDATE users SET name = ? WHERE id = ?")
        .bind(&new_name)
        .bind(&user_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let _ = crate::audit::log_action(
        &mut *conn,
        merchant_id,
        outlet_id,
        actor_id,
        "user.edit",
        "users",
        Some(user_id),
        Some(&format!("Nama pengguna diubah menjadi '{}'", new_name)),
    )
    .await;

    Ok(())
}

/// Assign role ke user di outlet saat ini.
#[tauri::command]
pub async fn assign_user_role(
    user_id: String,
    role_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    let actor_id = require_permission(pool.inner(), "role.assign").await?;
    let outlet_id = crate::auth::get_current_outlet(pool.inner())
        .await
        .map(|id| id.to_string())?;

    // Check duplicate active assignment
    let existing: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*) FROM user_outlet_roles
        WHERE user_id = ? AND outlet_id = ? AND role_id = ? AND status = 'ACTIVE'
        AND valid_until >= datetime('now')
        "#,
    )
    .bind(&user_id)
    .bind(&outlet_id)
    .bind(&role_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    if existing > 0 {
        return Err("Role ini sudah aktif untuk pengguna tersebut".to_string());
    }

    let uor_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO user_outlet_roles (id, user_id, outlet_id, role_id, valid_from, valid_until, status, assigned_by)
        VALUES (?, ?, ?, ?, datetime('now', '-1 day'), datetime('now', '+5 years'), 'ACTIVE', ?)
        "#,
    )
    .bind(&uor_id)
    .bind(&user_id)
    .bind(&outlet_id)
    .bind(&role_id)
    .bind(&actor_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let merchant_id: String = sqlx::query_scalar(
        "SELECT merchant_id FROM outlets WHERE id = ?",
    )
    .bind(&outlet_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let _ = crate::audit::log_action(
        &mut *conn,
        merchant_id,
        Some(outlet_id),
        actor_id,
        "role.assign",
        "user_outlet_roles",
        Some(uor_id.clone()),
        Some(&format!("Role role_id={} di-assign ke user_id={}", role_id, user_id)),
    )
    .await;

    Ok(uor_id)
}

/// Cabut role user (revoke assignment).
#[tauri::command]
pub async fn revoke_user_role(
    user_outlet_role_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let actor_id = require_permission(pool.inner(), "role.assign").await?;

    let row = sqlx::query(
        "SELECT user_id, outlet_id, role_id FROM user_outlet_roles WHERE id = ?",
    )
    .bind(&user_outlet_role_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .ok_or("Assignment role tidak ditemukan".to_string())?;

    let target_user: String = row.get("user_id");
    let outlet_id: String = row.get("outlet_id");
    let role_id: String = row.get("role_id");

    // Prevent revoking own role
    if target_user == actor_id {
        return Err("Anda tidak dapat mencabut role Anda sendiri".to_string());
    }

    sqlx::query(
        "UPDATE user_outlet_roles SET status = 'REVOKED', valid_until = datetime('now') WHERE id = ?",
    )
    .bind(&user_outlet_role_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let merchant_id: String = sqlx::query_scalar(
        "SELECT merchant_id FROM outlets WHERE id = ?",
    )
    .bind(&outlet_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let _ = crate::audit::log_action(
        &mut *conn,
        merchant_id,
        Some(outlet_id),
        actor_id,
        "role.revoke",
        "user_outlet_roles",
        Some(user_outlet_role_id),
        Some(&format!("Role role_id={} dicabut dari user_id={}", role_id, target_user)),
    )
    .await;

    Ok(())
}

/// Daftar semua role yang tersedia untuk outlet/merchant.
#[tauri::command]
pub async fn list_roles(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<RoleItem>, String> {
    require_permission(pool.inner(), "role.view").await?;
    let outlet_id = crate::auth::get_current_outlet(pool.inner())
        .await
        .map(|id| id.to_string())
        .unwrap_or_default();

    let merchant_id: String = sqlx::query_scalar(
        "SELECT merchant_id FROM outlets WHERE id = ?",
    )
    .bind(&outlet_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    let rows = sqlx::query(
        "SELECT id, name, system_role FROM roles WHERE merchant_id = ? ORDER BY name ASC",
    )
    .bind(&merchant_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let roles = rows
        .iter()
        .map(|r| RoleItem {
            id: r.get("id"),
            name: r.get("name"),
            system_role: r.get::<bool, _>("system_role"),
        })
        .collect();

    Ok(roles)
}

/// Unlock user yang terkunci karena terlalu banyak gagal login.
#[tauri::command]
pub async fn unlock_user(
    user_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let actor_id = require_permission(pool.inner(), "user.edit").await?;

    let row = sqlx::query("SELECT name, outlet_id FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Pengguna tidak ditemukan".to_string())?;
    let name: String = row.get("name");
    let outlet_id: Option<String> = row.get("outlet_id");
    let merchant_id: String = sqlx::query_scalar(
        "SELECT merchant_id FROM outlets WHERE id = ?",
    )
    .bind(outlet_id.as_deref().unwrap_or(""))
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    sqlx::query(
        "UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = ?",
    )
    .bind(&user_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    let _ = crate::audit::log_action(
        &mut *conn,
        merchant_id,
        outlet_id,
        actor_id,
        "user.unlock",
        "users",
        Some(user_id),
        Some(&format!("Kunci akun pengguna '{}' dibuka", name)),
    )
    .await;

    Ok(())
}
