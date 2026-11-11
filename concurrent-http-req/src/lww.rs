//! 0. Last-write-wins: no guard. The reference case that loses updates.
use crate::*;

pub async fn put(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(b): Json<ItemBody>,
) -> Result<Response, StatusCode> {
    let item = sqlx::query_as(&format!(
        "UPDATE items SET name = ?, qty = ?, version = version + 1 WHERE id = ? {RETURNING}"
    ))
    .bind(b.name)
    .bind(b.qty)
    .bind(id)
    .fetch_one(&s.db)
    .await
    .map_err(db_err)?;
    Ok(item_response(StatusCode::OK, item))
}
