/// Sentinel user id for anonymous/guest records in tables that require a
/// non-null user id (e.g. `personal_bests`). Real users start at id 1.
pub const GUEST_USER_ID: i64 = 0;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbSession {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub mode: String,
    pub duration: i64,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub started_at: i64,
    pub ended_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbKeystrokeLog {
    pub id: Option<i64>,
    pub session_id: i64,
    pub expected_char: String,
    pub actual_char: String,
    pub is_correct: bool,
    pub latency_ms: i64,
    pub position: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbText {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub char_count: i64,
    pub source: String, // "seed" | "user_paste" | "user_edit"
    pub created_by: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonalBest {
    pub user_id: i64, // 0 indicates guest
    pub mode: String,
    pub duration: i64,
    pub wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub session_id: i64,
    pub achieved_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PassageId {
    Passage(i64),
    LegacyText(i64),
}

impl PassageId {
    pub fn from_raw(raw: i64) -> Self {
        if raw < 0 {
            PassageId::LegacyText(-raw)
        } else {
            PassageId::Passage(raw)
        }
    }

    pub fn to_raw(self) -> i64 {
        match self {
            PassageId::Passage(id) => id,
            PassageId::LegacyText(id) => -id,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbPassage {
    pub id: i64,
    pub text_content: String,
    pub word_count: i64,
    pub category: String, // 'code', 'prose', 'quotes'
    pub is_custom: bool,
    pub created_at: i64,
}

impl DbPassage {
    pub fn passage_id(&self) -> PassageId {
        PassageId::from_raw(self.id)
    }
}
