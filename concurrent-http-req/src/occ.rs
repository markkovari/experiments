//! 1. Optimistic concurrency: ETag + If-Match (RFC 9110), 428 when missing (RFC 6585).
use crate::*;
use axum::http::HeaderMap;

pub async fn put(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(b): Json<ItemBody>,
) -> Result<Response, StatusCode> {
    let Some(if_match) = headers.get(header::IF_MATCH) else {
        return Err(StatusCode::PRECONDITION_REQUIRED);
    };
    // Unparseable tag can never match the current version.
    let expected: i64 = if_match
        .to_str()
        .ok()
        .and_then(|v| v.trim().trim_matches('"').parse().ok())
        .ok_or(StatusCode::PRECONDITION_FAILED)?;

    let item: Option<Item> = sqlx::query_as(&format!(
        "UPDATE items SET name = ?, qty = ?, version = version + 1 WHERE id = ? AND version = ? {RETURNING}"
    ))
    .bind(b.name)
    .bind(b.qty)
    .bind(id)
    .bind(expected)
    .fetch_optional(&s.db)
    .await
    .map_err(db_err)?;

    match item {
        Some(item) => Ok(item_response(StatusCode::OK, item)),
        None if exists(&s.db, id).await? => Err(StatusCode::PRECONDITION_FAILED),
        None => Err(StatusCode::NOT_FOUND),
    }
}
