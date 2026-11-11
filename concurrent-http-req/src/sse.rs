//! +. Change notifications: Server-Sent Events push every new version of an item,
//! so clients refresh before they write instead of finding out via a 412/409.
use crate::*;
use axum::extract::Query;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::{self, Stream};
use std::convert::Infallible;

#[derive(Deserialize)]
pub struct SseQuery {
    /// Close the stream after this many events (handy for tests and one-shot clients).
    limit: Option<usize>,
}

pub async fn events(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<SseQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    if !exists(&s.db, id).await? {
        return Err(StatusCode::NOT_FOUND);
    }
    let start = (s.db, 0i64, q.limit.unwrap_or(usize::MAX));
    // shortcut: polls the row every 100 ms (sees writes from every endpoint and the queue worker);
    // switch to an in-process broadcast or sqlite update_hook if many clients subscribe.
    let stream = stream::unfold(start, move |(db, last, left)| async move {
        if left == 0 {
            return None;
        }
        loop {
            let item: Item = sqlx::query_as("SELECT id, name, qty, version FROM items WHERE id = ?")
                .bind(id)
                .fetch_optional(&db)
                .await
                .ok()??; // db error or item gone: end the stream
            if item.version != last {
                let v = item.version;
                let ev = Event::default().event("item").id(v.to_string()).json_data(&item).ok()?;
                return Some((Ok(ev), (db, v, left - 1)));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
