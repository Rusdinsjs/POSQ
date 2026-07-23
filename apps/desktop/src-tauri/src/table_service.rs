use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    Open,
    OrderPlaced,
    CourseServed,
    SplitBill,
    Paid,
    Closed,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Open => "open",
            SessionState::OrderPlaced => "order_placed",
            SessionState::CourseServed => "course_served",
            SessionState::SplitBill => "split_bill",
            SessionState::Paid => "paid",
            SessionState::Closed => "closed",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "open" => Ok(SessionState::Open),
            "order_placed" => Ok(SessionState::OrderPlaced),
            "course_served" => Ok(SessionState::CourseServed),
            "split_bill" => Ok(SessionState::SplitBill),
            "paid" => Ok(SessionState::Paid),
            "closed" => Ok(SessionState::Closed),
            _ => Err(format!("Unknown session state: {}", s)),
        }
    }

    pub fn can_transition_to(&self, next: &SessionState) -> bool {
        match (self, next) {
            (SessionState::Open, SessionState::OrderPlaced) => true,
            (SessionState::Open, SessionState::Closed) => true, // Empty table cancel
            (SessionState::OrderPlaced, SessionState::CourseServed) => true,
            (SessionState::OrderPlaced, SessionState::SplitBill) => true,
            (SessionState::OrderPlaced, SessionState::Paid) => true,
            (SessionState::CourseServed, SessionState::SplitBill) => true,
            (SessionState::CourseServed, SessionState::Paid) => true,
            (SessionState::SplitBill, SessionState::Paid) => true,
            (SessionState::Paid, SessionState::Closed) => true,
            _ => false,
        }
    }
}

pub async fn transition_session_state(
    pool: &Pool<Sqlite>,
    session_id: &str,
    target_state: SessionState,
) -> Result<(), String> {
    let row = sqlx::query("SELECT status FROM dining_sessions WHERE id = ?")
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    let current = match row {
        Some(r) => SessionState::parse(&r.get::<String, _>("status"))?,
        None => return Err(format!("SESSION_NOT_FOUND: Dining session '{}' tidak ditemukan", session_id)),
    };

    if !current.can_transition_to(&target_state) {
        return Err(format!(
            "INVALID_STATE_TRANSITION: Tidak dapat mengubah status sesi dari '{:?}' ke '{:?}'",
            current, target_state
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE dining_sessions SET status = ?, updated_at = ? WHERE id = ?")
        .bind(target_state.as_str())
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Tauri commands
#[tauri::command]
pub async fn update_session_state_cmd(
    pool: tauri::State<'_, Pool<Sqlite>>,
    session_id: String,
    target_state: String,
) -> Result<(), String> {
    let next_state = SessionState::parse(&target_state)?;
    transition_session_state(&pool, &session_id, next_state).await
}
