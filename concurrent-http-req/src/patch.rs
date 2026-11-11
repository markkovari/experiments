//! 4. Partial + relative updates: only touch the fields sent, and `qty_inc` adds
//! instead of overwriting, so edits to different fields (or two increments) both survive.
use crate::*;

#[derive(Deserialize, Serialize)]
pub struct PatchBody {
    pub name: Option<String>,
    pub qty: Option<i64>,
    pub qty_inc: Option<i64>,
    /// Optional guard, like a JSON Patch `test` op on the version.
    pub expected_version: Option<i64>,
}

/// Shared with the command queue worker. `None` = not found or guard failed.
pub async fn apply(db: &SqlitePool, id: i64, b: &PatchBody) -> Result<Option<Item>, sqlx::Error> {
    sqlx::query_as(&format!(
        "UPDATE items SET
           name = COALESCE(?1, name),
           qty = COALESCE(?2, qty) + COALESCE(?3, 0),
           version = version + 1
         WHERE id = ?4 AND (?5 IS NULL OR version = ?5)
         {RETURNING}"
    ))
    .bind(&b.name)
    .bind(b.qty)
    .bind(b.qty_inc)
    .bind(id)
    .bind(b.expected_version)
    .fetch_optional(db)
    .await
}

pub async fn patch(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(b): Json<PatchBody>,
) -> Result<Response, StatusCode> {
    match apply(&s.db, id, &b).await.map_err(db_err)? {
        Some(item) => Ok(item_response(StatusCode::OK, item)),
        None if exists(&s.db, id).await? => Err(StatusCode::CONFLICT),
        None => Err(StatusCode::NOT_FOUND),
    }
}
