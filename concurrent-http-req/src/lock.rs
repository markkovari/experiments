//! 3. Pessimistic lock with a lease: acquire a token, write with it, release it.
//! Expired leases are taken over, so a crashed client cannot block an item forever.
use crate::*;
use axum::extract::Query;
use axum::http::HeaderMap;

const LOCK_TOKEN: &str = "lock-token";

#[derive(Deserialize)]
pub struct LockQuery {
    ttl_secs: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Lease {
    token: String,
    expires_at: i64,
}

fn token(headers: &HeaderMap) -> &str {
    headers.get(LOCK_TOKEN).and_then(|v| v.to_str().ok()).unwrap_or("")
}

pub async fn acquire(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<LockQuery>,
) -> Result<Response, StatusCode> {
    let ttl = q.ttl_secs.unwrap_or(30).clamp(1, 300);
    // One atomic statement: insert, or take over only if the current lease has expired.
    let lease: Option<Lease> = sqlx::query_as(
        "INSERT INTO locks (item_id, token, expires_at)
         VALUES (?1, lower(hex(randomblob(16))), unixepoch() + ?2)
         ON CONFLICT (item_id) DO UPDATE
           SET token = excluded.token, expires_at = excluded.expires_at
           WHERE locks.expires_at <= unixepoch()
         RETURNING token, expires_at",
    )
    .bind(id)
    .bind(ttl)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| match e.as_database_error() {
        Some(d) if d.is_foreign_key_violation() => StatusCode::NOT_FOUND,
        _ => db_err(e),
    })?;

    let lease = lease.ok_or(StatusCode::LOCKED)?;
    let hv = HeaderValue::from_str(&lease.token).unwrap();
    Ok((StatusCode::CREATED, [(LOCK_TOKEN, hv)], Json(lease)).into_response())
}

pub async fn release(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let n = sqlx::query("DELETE FROM locks WHERE item_id = ? AND token = ?")
        .bind(id)
        .bind(token(&headers))
        .execute(&s.db)
        .await
        .map_err(db_err)?
        .rows_affected();
    if n == 1 { Ok(StatusCode::NO_CONTENT) } else { Err(StatusCode::LOCKED) }
}

pub async fn put(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(b): Json<ItemBody>,
) -> Result<Response, StatusCode> {
    let item: Option<Item> = sqlx::query_as(&format!(
        "UPDATE items SET name = ?1, qty = ?2, version = version + 1
         WHERE id = ?3 AND EXISTS (
           SELECT 1 FROM locks WHERE item_id = ?3 AND token = ?4 AND expires_at > unixepoch())
         {RETURNING}"
    ))
    .bind(b.name)
    .bind(b.qty)
    .bind(id)
    .bind(token(&headers))
    .fetch_optional(&s.db)
    .await
    .map_err(db_err)?;

    match item {
        Some(item) => Ok(item_response(StatusCode::OK, item)),
        None if exists(&s.db, id).await? => Err(StatusCode::LOCKED),
        None => Err(StatusCode::NOT_FOUND),
    }
}
