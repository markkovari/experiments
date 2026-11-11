//! 5. Server-side three-way merge: the client sends the version it edited (base) and
//! its full edited document. Fields only one side changed are merged automatically;
//! fields both sides changed differently come back as a 409 with the conflict details.
use crate::*;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct MergeBody {
    base_version: i64,
    name: String,
    qty: i64,
}

/// Merge one field. Records a conflict and keeps theirs when both sides changed it differently.
fn pick<T: PartialEq + Serialize>(field: &str, base: T, mine: T, theirs: T, conflicts: &mut Vec<Value>) -> T {
    if mine == base || mine == theirs {
        theirs
    } else if theirs == base {
        mine
    } else {
        conflicts.push(json!({ "field": field, "base": base, "mine": mine, "theirs": theirs }));
        theirs
    }
}

pub async fn put(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(b): Json<MergeBody>,
) -> Result<Response, Response> {
    let err = |c: StatusCode| c.into_response();
    let base: (String, i64) = sqlx::query_as("SELECT name, qty FROM item_history WHERE item_id = ? AND version = ?")
        .bind(id)
        .bind(b.base_version)
        .fetch_one(&s.db)
        .await
        .map_err(|e| err(db_err(e)))?;

    // Retry if someone writes between our read of "theirs" and our guarded update.
    loop {
        let cur: Item = sqlx::query_as("SELECT id, name, qty, version FROM items WHERE id = ?")
            .bind(id)
            .fetch_one(&s.db)
            .await
            .map_err(|e| err(db_err(e)))?;

        let mut conflicts = Vec::new();
        let name = pick("name", base.0.clone(), b.name.clone(), cur.name.clone(), &mut conflicts);
        let qty = pick("qty", base.1, b.qty, cur.qty, &mut conflicts);
        if !conflicts.is_empty() {
            return Err((StatusCode::CONFLICT, Json(json!({ "current": cur, "conflicts": conflicts }))).into_response());
        }

        let merged: Option<Item> = sqlx::query_as(&format!(
            "UPDATE items SET name = ?, qty = ?, version = version + 1 WHERE id = ? AND version = ? {RETURNING}"
        ))
        .bind(name)
        .bind(qty)
        .bind(id)
        .bind(cur.version)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| err(db_err(e)))?;
        if let Some(item) = merged {
            return Ok(item_response(StatusCode::OK, item));
        }
    }
}
