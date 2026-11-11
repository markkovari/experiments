//! 2. If-Unmodified-Since: like If-Match, but compares the Last-Modified timestamp.
//! Weakness: HTTP-dates have 1 s resolution, so two writes in the same second both pass.
use crate::*;
use axum::http::HeaderMap;

pub async fn put(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(b): Json<ItemBody>,
) -> Result<Response, StatusCode> {
    let Some(h) = headers.get(header::IF_UNMODIFIED_SINCE) else {
        return Err(StatusCode::PRECONDITION_REQUIRED);
    };
    let since = h
        .to_str()
        .ok()
        .and_then(|v| httpdate::parse_http_date(v).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let since = since.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;

    let item: Option<Item> = sqlx::query_as(&format!(
        "UPDATE items SET name = ?1, qty = ?2, version = version + 1
         WHERE id = ?3 AND COALESCE((SELECT at FROM item_history h
                            WHERE h.item_id = items.id AND h.version = items.version), 0) <= ?4
         {RETURNING}"
    ))
    .bind(b.name)
    .bind(b.qty)
    .bind(id)
    .bind(since)
    .fetch_optional(&s.db)
    .await
    .map_err(db_err)?;

    match item {
        Some(item) => {
            let mut res = item_response(StatusCode::OK, item);
            res.headers_mut().insert(header::LAST_MODIFIED, last_modified(&s.db, id).await?);
            Ok(res)
        }
        None if exists(&s.db, id).await? => Err(StatusCode::PRECONDITION_FAILED),
        None => Err(StatusCode::NOT_FOUND),
    }
}
