//! JobRunner Connect-RPC server.
//!
//! Demonstrates two ways a Connect client observes a multi-step job:
//!   * `RunJobStreaming` — server-streaming RPC; the server pushes a
//!     `JobUpdate` per step as the work runs.
//!   * `StartJob` + `GetJobStatus` — a unary pair the client polls until
//!     the job reports DONE.
//!
//! Run with: `cargo run` (listens on 127.0.0.1:8080).

// The generated trait methods return `impl Encodable<T>`; our concrete
// `ServiceResult<T>` refines that. This is the documented usage pattern.
#![allow(refining_impl_trait)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use connectrpc::{RequestContext, Response, Router, ServiceRequest, ServiceResult, ServiceStream};
use tower_http::cors::CorsLayer;

pub mod proto {
    connectrpc::include_generated!();
}

use proto::jobrunner::v1::*;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// In-memory record of a polled job's progress.
#[derive(Clone)]
struct JobState {
    phase: JobPhase,
    step: i32,
    total: i32,
    message: String,
}

/// Shared application state: the job store backing the polling RPCs.
#[derive(Clone, Default)]
struct AppState {
    jobs: Arc<Mutex<HashMap<String, JobState>>>,
}

struct JobRunner {
    state: AppState,
}

/// Clamp a requested step count into a sane demo range.
fn clamp_steps(requested: i32) -> i32 {
    requested.clamp(1, 50)
}

impl JobRunnerService for JobRunner {
    /// Server streaming: emit a `JobUpdate` per step, then a final
    /// update with `done = true`. A short sleep between steps makes the
    /// streaming visible in the UI.
    async fn run_job_streaming(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, JobRequest>,
    ) -> ServiceResult<ServiceStream<JobUpdate>> {
        // Accessors return borrowed views tied to `request`; copy out
        // owned values before they escape into the returned stream.
        // Fields are borrowed-view values; `.to_string()` owns the label
        // so it can escape into the returned stream.
        let label = request.label.unwrap_or_default().to_string();
        let total = clamp_steps(request.steps.unwrap_or(0));

        // unfold drives one update per tick; state is the next step index.
        let stream = futures::stream::unfold(0i32, move |step| {
            let label = label.clone();
            async move {
                if step > total {
                    return None;
                }
                // Pace the stream so updates arrive over time, not at once.
                tokio::time::sleep(Duration::from_millis(400)).await;
                let done = step == total;
                let message = if done {
                    format!("'{label}' complete")
                } else {
                    format!("'{label}' step {step}/{total}")
                };
                let update = JobUpdate {
                    step: Some(step),
                    total: Some(total),
                    message: Some(message),
                    done: Some(done),
                    ..Default::default()
                };
                Some((Ok(update), step + 1))
            }
        });

        Response::stream_ok(stream)
    }

    /// Kick off a background job and return a handle. A spawned task
    /// advances the stored progress over time; the client polls
    /// `GetJobStatus` to observe it.
    async fn start_job(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, JobRequest>,
    ) -> ServiceResult<JobHandle> {
        let label = request.label.unwrap_or_default().to_string();
        let total = clamp_steps(request.steps.unwrap_or(0));
        let id = uuid::Uuid::new_v4().to_string();

        self.state.jobs.lock().unwrap().insert(
            id.clone(),
            JobState {
                phase: JobPhase::Running,
                step: 0,
                total,
                message: format!("'{label}' queued"),
            },
        );

        let jobs = self.state.jobs.clone();
        let task_id = id.clone();
        tokio::spawn(async move {
            for step in 1..=total {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let mut guard = jobs.lock().unwrap();
                let Some(job) = guard.get_mut(&task_id) else {
                    return;
                };
                job.step = step;
                if step == total {
                    job.phase = JobPhase::Done;
                    job.message = format!("'{label}' complete");
                } else {
                    job.message = format!("'{label}' step {step}/{total}");
                }
            }
        });

        Response::ok(JobHandle {
            id: Some(id),
            ..Default::default()
        })
    }

    /// Read the current status of a started job.
    async fn get_job_status(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, JobHandle>,
    ) -> ServiceResult<JobStatus> {
        let id = request.id.unwrap_or_default().to_string();
        let guard = self.state.jobs.lock().unwrap();

        let status = match guard.get(&id) {
            Some(job) => JobStatus {
                id: Some(id),
                phase: Some(job.phase.into()),
                step: Some(job.step),
                total: Some(job.total),
                message: Some(job.message.clone()),
                ..Default::default()
            },
            None => JobStatus {
                id: Some(id),
                phase: Some(JobPhase::Unspecified.into()),
                step: Some(0),
                total: Some(0),
                message: Some("unknown job".to_string()),
                ..Default::default()
            },
        };

        Response::ok(status)
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // 8088 (not 8080) to avoid clashing with common local services.
    let addr: std::net::SocketAddr = "127.0.0.1:8088".parse()?;

    let service = Arc::new(JobRunner {
        state: AppState::default(),
    });
    let router = service.register(Router::new());

    // Permissive CORS so a production web build can call cross-origin.
    // (The dev setup uses a Vite proxy and never needs this.)
    let app = router.into_axum_router().layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("JobRunnerService listening on http://{addr}");

    axum::serve(listener, app).await?;
    Ok(())
}
