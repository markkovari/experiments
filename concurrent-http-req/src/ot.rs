//! 9. Operational transform on a plain-text document.
//! A client sends one op made against `base_version`. The server transforms it against
//! every op committed since then, so it still means what the client intended, then applies it.
//! Positions are in characters. On equal insert positions, the op committed first goes first.
use crate::*;
use serde_json::json;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Insert { pos: usize, text: String },
    Delete { pos: usize, len: usize },
}

/// Rewrite `op` so it applies after `against`. Can split a delete into two.
fn transform(op: Op, against: &Op) -> Vec<Op> {
    use Op::*;
    match (op, against) {
        (Insert { pos, text }, Insert { pos: q, text: t }) => {
            let pos = if *q <= pos { pos + t.chars().count() } else { pos };
            vec![Insert { pos, text }]
        }
        (Insert { pos, text }, Delete { pos: q, len: m }) => {
            let pos = if pos <= *q { pos } else if pos >= q + m { pos - m } else { *q };
            vec![Insert { pos, text }]
        }
        (Delete { pos, len }, Insert { pos: q, text: t }) => {
            let s = t.chars().count();
            if *q <= pos {
                vec![Delete { pos: pos + s, len }]
            } else if *q >= pos + len {
                vec![Delete { pos, len }]
            } else {
                // Concurrent insert landed inside our range: delete around it, later part first.
                vec![Delete { pos: q + s, len: pos + len - q }, Delete { pos, len: q - pos }]
            }
        }
        (Delete { pos, len }, Delete { pos: q, len: m }) => {
            if q + m <= pos {
                vec![Delete { pos: pos - m, len }]
            } else if *q >= pos + len {
                vec![Delete { pos, len }]
            } else {
                let overlap = (pos + len).min(q + m) - pos.max(*q);
                let len = len - overlap;
                if len == 0 { vec![] } else { vec![Delete { pos: pos.min(*q), len }] }
            }
        }
    }
}

fn apply(text: &str, op: &Op) -> Option<String> {
    let mut chars: Vec<char> = text.chars().collect();
    match op {
        Op::Insert { pos, text } => {
            if *pos > chars.len() {
                return None;
            }
            chars.splice(pos..pos, text.chars());
        }
        Op::Delete { pos, len } => {
            if pos + len > chars.len() {
                return None;
            }
            chars.drain(*pos..pos + len);
        }
    }
    Some(chars.into_iter().collect())
}

#[derive(Deserialize)]
pub struct NewDoc {
    text: String,
}

#[derive(Deserialize)]
pub struct Submit {
    base_version: i64,
    op: Op,
}

pub async fn create(State(s): State<AppState>, Json(d): Json<NewDoc>) -> Result<Response, StatusCode> {
    let (id, version): (i64, i64) = sqlx::query_as("INSERT INTO ot_docs (text) VALUES (?) RETURNING id, version")
        .bind(&d.text)
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    let loc = HeaderValue::from_str(&format!("/ot/docs/{id}")).unwrap();
    let body = Json(json!({ "id": id, "version": version, "text": d.text }));
    Ok((StatusCode::CREATED, [(header::LOCATION, loc)], body).into_response())
}

pub async fn read(State(s): State<AppState>, Path(id): Path<i64>) -> Result<Response, StatusCode> {
    let (text, version): (String, i64) = sqlx::query_as("SELECT text, version FROM ot_docs WHERE id = ?")
        .bind(id)
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    let etag = HeaderValue::from_str(&format!("\"{version}\"")).unwrap();
    Ok(([(header::ETAG, etag)], Json(json!({ "id": id, "version": version, "text": text }))).into_response())
}

pub async fn submit(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<Submit>,
) -> Result<Response, StatusCode> {
    // Retry if another op commits between our read and our guarded write.
    loop {
        let (text, version): (String, i64) = sqlx::query_as("SELECT text, version FROM ot_docs WHERE id = ?")
            .bind(id)
            .fetch_one(&s.db)
            .await
            .map_err(db_err)?;
        if req.base_version < 0 || req.base_version > version {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
        let later: Vec<String> =
            sqlx::query_scalar("SELECT ops FROM ot_ops WHERE doc_id = ? AND version > ? ORDER BY version")
                .bind(id)
                .bind(req.base_version)
                .fetch_all(&s.db)
                .await
                .map_err(db_err)?;

        let mut ops = vec![req.op.clone()];
        for committed in &later {
            let committed: Vec<Op> = serde_json::from_str(committed).expect("ops were serialized by submit");
            for a in &committed {
                ops = ops.into_iter().flat_map(|o| transform(o, a)).collect();
            }
        }
        let mut new_text = text;
        for o in &ops {
            new_text = apply(&new_text, o).ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
        }

        let mut tx = s.db.begin().await.map_err(db_err)?;
        let n = sqlx::query("UPDATE ot_docs SET text = ?, version = version + 1 WHERE id = ? AND version = ?")
            .bind(&new_text)
            .bind(id)
            .bind(version)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?
            .rows_affected();
        if n == 0 {
            continue; // dropped tx rolls back
        }
        sqlx::query("INSERT INTO ot_ops (doc_id, version, ops) VALUES (?, ?, ?)")
            .bind(id)
            .bind(version + 1)
            .bind(serde_json::to_string(&ops).unwrap())
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        tx.commit().await.map_err(db_err)?;
        let body = json!({ "version": version + 1, "text": new_text, "applied": ops });
        return Ok((StatusCode::CREATED, Json(body)).into_response());
    }
}
