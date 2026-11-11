//! 7. Command queue: writes become commands (202 Accepted), applied one at a time
//! by a single background worker, so they never race each other.
use crate::patch::{PatchBody, apply};
use crate::*;
use tokio::sync::mpsc;

#[derive(Serialize, sqlx::FromRow)]
pub struct Command {
    id: i64,
    item_id: i64,
    status: String,
    error: Option<String>,
}

// shortcut: one global worker serializes all items; shard per item_id if throughput matters.
pub async fn spawn_worker(db: SqlitePool) -> UnboundedSender<i64> {
    let (tx, mut rx) = mpsc::unbounded_channel::<i64>();
    // Commands accepted before a restart are still pending: re-enqueue them.
    let pending: Vec<i64> = sqlx::query_scalar("SELECT id FROM commands WHERE status = 'pending' ORDER BY id")
        .fetch_all(&db)
        .await
        .expect("load pending commands");
    for id in pending {
        tx.send(id).unwrap();
    }
    tokio::spawn(async move {
        while let Some(id) = rx.recv().await {
            if let Err(e) = run(&db, id).await {
                eprintln!("command {id}: {e}");
            }
        }
    });
    tx
}

async fn run(db: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    let (item_id, payload): (i64, String) =
        sqlx::query_as("SELECT item_id, payload FROM commands WHERE id = ?").bind(id).fetch_one(db).await?;
    let body: PatchBody = serde_json::from_str(&payload).expect("payload was serialized by submit");
    let error = match apply(db, item_id, &body).await? {
        Some(_) => None,
        None if body.expected_version.is_some() => Some("version mismatch or item missing"),
        None => Some("item not found"),
    };
    sqlx::query("UPDATE commands SET status = ?, error = ? WHERE id = ?")
        .bind(if error.is_some() { "failed" } else { "done" })
        .bind(error)
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn submit(
    State(s): State<AppState>,
    Path(item_id): Path<i64>,
    Json(b): Json<PatchBody>,
) -> Result<Response, StatusCode> {
    let cmd: Command = sqlx::query_as(
        "INSERT INTO commands (item_id, payload) VALUES (?, ?) RETURNING id, item_id, status, error",
    )
    .bind(item_id)
    .bind(serde_json::to_string(&b).unwrap())
    .fetch_one(&s.db)
    .await
    .map_err(db_err)?;
    s.queue.send(cmd.id).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let loc = HeaderValue::from_str(&format!("/commands/{}", cmd.id)).unwrap();
    Ok((StatusCode::ACCEPTED, [(header::LOCATION, loc)], Json(cmd)).into_response())
}

pub async fn status(State(s): State<AppState>, Path(id): Path<i64>) -> Result<Json<Command>, StatusCode> {
    sqlx::query_as("SELECT id, item_id, status, error FROM commands WHERE id = ?")
        .bind(id)
        .fetch_one(&s.db)
        .await
        .map(Json)
        .map_err(db_err)
}
