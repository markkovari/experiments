//! 8. State-based CRDTs: a PN-counter and a last-writer-wins register.
//! Each replica (client) sends its own running totals; merging takes the max per replica,
//! so sends can be repeated, reordered or delayed and every order converges to the same value.
use crate::*;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct Register {
    value: String,
    ts: i64,
}

#[derive(Deserialize)]
pub struct ReplicaState {
    replica: String,
    /// Total increments this replica has ever made (monotonic).
    #[serde(default)]
    p: i64,
    /// Total decrements this replica has ever made (monotonic).
    #[serde(default)]
    n: i64,
    name: Option<Register>,
}

async fn state(db: &SqlitePool, key: &str) -> Result<Option<Value>, StatusCode> {
    let (value, replicas): (i64, i64) =
        sqlx::query_as("SELECT COALESCE(SUM(p) - SUM(n), 0), COUNT(*) FROM crdt_counters WHERE key = ?")
            .bind(key)
            .fetch_one(db)
            .await
            .map_err(db_err)?;
    let name: Option<String> = sqlx::query_scalar("SELECT value FROM crdt_registers WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await
        .map_err(db_err)?;
    if replicas == 0 && name.is_none() {
        return Ok(None);
    }
    Ok(Some(json!({ "value": value, "name": name, "replicas": replicas })))
}

pub async fn read(State(s): State<AppState>, Path(key): Path<String>) -> Result<Json<Value>, StatusCode> {
    state(&s.db, &key).await?.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn merge(
    State(s): State<AppState>,
    Path(key): Path<String>,
    Json(r): Json<ReplicaState>,
) -> Result<Json<Value>, StatusCode> {
    if r.p < 0 || r.n < 0 || r.replica.is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let mut tx = s.db.begin().await.map_err(db_err)?;
    sqlx::query(
        "INSERT INTO crdt_counters (key, replica, p, n) VALUES (?, ?, ?, ?)
         ON CONFLICT (key, replica) DO UPDATE SET p = MAX(p, excluded.p), n = MAX(n, excluded.n)",
    )
    .bind(&key)
    .bind(&r.replica)
    .bind(r.p)
    .bind(r.n)
    .execute(&mut *tx)
    .await
    .map_err(db_err)?;
    if let Some(reg) = &r.name {
        // Highest (ts, replica) wins; the replica id breaks timestamp ties deterministically.
        sqlx::query(
            "INSERT INTO crdt_registers (key, value, ts, replica) VALUES (?, ?, ?, ?)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value, ts = excluded.ts, replica = excluded.replica
             WHERE (excluded.ts, excluded.replica) > (crdt_registers.ts, crdt_registers.replica)",
        )
        .bind(&key)
        .bind(&reg.value)
        .bind(reg.ts)
        .bind(&r.replica)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
    }
    tx.commit().await.map_err(db_err)?;
    Ok(Json(state(&s.db, &key).await?.expect("just written")))
}
