mod crdt;
mod es;
mod ius;
mod lock;
mod lww;
mod merge;
mod occ;
mod ot;
mod patch;
mod queue;
mod sse;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::{str::FromStr, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub queue: UnboundedSender<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub qty: i64,
    pub version: i64,
}

/// Full replacement body (PUT / POST).
#[derive(Deserialize)]
pub struct ItemBody {
    pub name: String,
    pub qty: i64,
}

pub const RETURNING: &str = "RETURNING id, name, qty, version";

/// Every item response carries its version as a strong ETag.
pub fn item_response(status: StatusCode, item: Item) -> Response {
    let etag = HeaderValue::from_str(&format!("\"{}\"", item.version)).unwrap();
    (status, [(header::ETAG, etag)], Json(item)).into_response()
}

pub fn db_err(e: sqlx::Error) -> StatusCode {
    match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        e => {
            eprintln!("db error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Used after a guarded UPDATE matched nothing: was it a missing row or a failed guard?
pub async fn exists(db: &SqlitePool, id: i64) -> Result<bool, StatusCode> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM items WHERE id = ?)")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(db_err)
}

/// When the item's current version was written, as an HTTP-date (whole seconds).
pub async fn last_modified(db: &SqlitePool, id: i64) -> Result<HeaderValue, StatusCode> {
    // Items written before item_history existed have no row: report them as epoch (oldest).
    let at: i64 = sqlx::query_scalar(
        "SELECT COALESCE(h.at, 0) FROM items i
         LEFT JOIN item_history h ON h.item_id = i.id AND h.version = i.version WHERE i.id = ?",
    )
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(db_err)?;
    let t = std::time::UNIX_EPOCH + Duration::from_secs(at as u64);
    Ok(HeaderValue::from_str(&httpdate::fmt_http_date(t)).unwrap())
}

async fn create(State(s): State<AppState>, Json(b): Json<ItemBody>) -> Result<Response, StatusCode> {
    let item: Item = sqlx::query_as(&format!("INSERT INTO items (name, qty) VALUES (?, ?) {RETURNING}"))
        .bind(b.name)
        .bind(b.qty)
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    let loc = HeaderValue::from_str(&format!("/items/{}", item.id)).unwrap();
    let mut res = item_response(StatusCode::CREATED, item);
    res.headers_mut().insert(header::LOCATION, loc);
    Ok(res)
}

async fn read(State(s): State<AppState>, Path(id): Path<i64>) -> Result<Response, StatusCode> {
    let item = sqlx::query_as("SELECT id, name, qty, version FROM items WHERE id = ?")
        .bind(id)
        .fetch_one(&s.db)
        .await
        .map_err(db_err)?;
    let mut res = item_response(StatusCode::OK, item);
    res.headers_mut().insert(header::LAST_MODIFIED, last_modified(&s.db, id).await?);
    Ok(res)
}

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:app.db".into());
    let opts = SqliteConnectOptions::from_str(&url)
        .expect("bad DATABASE_URL")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let db = SqlitePool::connect_with(opts).await.expect("open db");
    sqlx::raw_sql(include_str!("schema.sql")).execute(&db).await.expect("schema");

    let queue = queue::spawn_worker(db.clone()).await;
    let state = AppState { db, queue };

    let app = Router::new()
        .route("/items", post(create))
        .route("/items/{id}", get(read))
        .route("/items/{id}/events", get(sse::events))
        .route("/lww/items/{id}", put(lww::put))
        .route("/occ/items/{id}", put(occ::put))
        .route("/ius/items/{id}", put(ius::put))
        .route("/merge/items/{id}", put(merge::put))
        .route("/crdt/{key}", get(crdt::read).post(crdt::merge))
        .route("/ot/docs", post(ot::create))
        .route("/ot/docs/{id}", get(ot::read))
        .route("/ot/docs/{id}/ops", post(ot::submit))
        .route("/lock/items/{id}", put(lock::put))
        .route("/lock/items/{id}/lock", post(lock::acquire).delete(lock::release))
        .route("/patch/items/{id}", axum::routing::patch(patch::patch))
        .route("/queue/items/{id}/commands", post(queue::submit))
        .route("/commands/{id}", get(queue::status))
        .route("/es/streams/{id}", get(es::read))
        .route("/es/streams/{id}/events", post(es::append))
        .with_state(state);

    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    println!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
