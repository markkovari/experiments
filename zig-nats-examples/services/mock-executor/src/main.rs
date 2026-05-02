use futures::StreamExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "mock_executor=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Mock Executor Service...");

    let settings = AppConfig::load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    let nats_client = async_nats::connect(&settings.nats.url).await?;
    let js = async_nats::jetstream::new(nats_client.clone());

    let stream = js.get_stream("ACTIONS_DISPATCH").await?;
    let consumer = stream.get_or_create_consumer("executor-worker", async_nats::jetstream::consumer::pull::Config {
        durable_name: Some("executor-worker".to_string()),
        ..Default::default()
    }).await?;

    let mut messages = consumer.messages().await?;
    tracing::info!("Waiting for actions...");

    while let Some(Ok(message)) = messages.next().await {
        let payload: serde_json::Value = serde_json::from_slice(&message.payload)?;
        let execution_id = payload["execution_id"].as_str().unwrap_or("unknown");
        let action_type = message.subject.split('.').last().unwrap_or("unknown");

        tracing::info!("Executing action {} (type: {})", execution_id, action_type);

        // Simulate work
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let outcome = serde_json::json!({
            "execution_id": execution_id,
            "status": "Completed",
            "result": {
                "message": format!("Successfully processed {} at {}", action_type, chrono::Utc::now()),
                "data": payload["payload"]
            }
        });

        let outcome_subject = format!("action.outcome.{}", action_type);
        js.publish(outcome_subject, serde_json::to_vec(&outcome)?.into()).await?;
        
        tracing::info!("Action {} completed and outcome published", execution_id);

        message.ack().await.map_err(|e| anyhow::anyhow!("Ack failed: {}", e))?;
    }

    Ok(())
}
