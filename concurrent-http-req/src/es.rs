//! 6. Event sourcing with optimistic append: the client says which version it saw;
//! the append only succeeds if that is still the stream's latest version.
use crate::*;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct Append {
    expected_version: i64,
    #[serde(rename = "type")]
    kind: String,
    data: Value,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Event {
    version: i64,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    kind: String,
    #[sqlx(json)]
    data: Value,
}

async fn current_version(db: &SqlitePool, stream: &str) -> Result<i64, StatusCode> {
    sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = ?")
        .bind(stream)
        .fetch_one(db)
        .await
        .map_err(db_err)
}

pub async fn append(
    State(s): State<AppState>,
    Path(stream): Path<String>,
    Json(a): Json<Append>,
) -> Result<Response, Response> {
    // Check-and-insert in one statement; the primary key is the backstop.
    let res = sqlx::query(
        "INSERT INTO events (stream_id, version, type, data)
         SELECT ?1, ?2 + 1, ?3, ?4
         WHERE (SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = ?1) = ?2",
    )
    .bind(&stream)
    .bind(a.expected_version)
    .bind(&a.kind)
    .bind(a.data.to_string())
    .execute(&s.db)
    .await;

    let inserted = match res {
        Ok(r) => r.rows_affected() == 1,
        Err(e) if e.as_database_error().is_some_and(|d| d.is_unique_violation()) => false,
        Err(e) => return Err(db_err(e).into_response()),
    };
    if inserted {
        let v = a.expected_version + 1;
        return Ok((StatusCode::CREATED, Json(json!({ "version": v }))).into_response());
    }
    let current = current_version(&s.db, &stream).await.map_err(IntoResponse::into_response)?;
    Err((StatusCode::CONFLICT, Json(json!({ "current_version": current }))).into_response())
}

pub async fn read(State(s): State<AppState>, Path(stream): Path<String>) -> Result<Response, StatusCode> {
    let events: Vec<Event> =
        sqlx::query_as("SELECT version, type, data FROM events WHERE stream_id = ? ORDER BY version")
            .bind(&stream)
            .fetch_all(&s.db)
            .await
            .map_err(db_err)?;
    if events.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }
    let version = events.last().unwrap().version;
    let etag = HeaderValue::from_str(&format!("\"{version}\"")).unwrap();
    Ok(([(header::ETAG, etag)], Json(json!({ "version": version, "events": events }))).into_response())
}
