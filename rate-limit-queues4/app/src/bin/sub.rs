use async_nats::jetstream::{
    self, AckKind,
    consumer::{AckPolicy, DeliverPolicy, PullConsumer},
};
use std::{
    env::{self, args},
    str::from_utf8,
};
use time::{Duration, OffsetDateTime};

use futures::StreamExt;
use log::{debug, info};

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let consumer_name = match args().skip(1).take(1).next() {
        Some(name) => format!("events.{}", name),
        _ => "events.low".to_string(),
    };

    env_logger::init();

    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let client = async_nats::connect(nats_url).await?;

    let jetstream = jetstream::new(client);

    let stream_name = String::from("EVENTS");

    // Create a stream and a consumer.
    // We can chain the methods.
    // First we create a stream and bind to it.

    let stream: jetstream::stream::Stream = jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: stream_name,
            subjects: vec!["events.>".into()],
            ..Default::default()
        })
        .await?;

    let config = jetstream::consumer::pull::Config {
        name: Some(format!("{consumer_name}_consumer")),
        durable_name: Some(format!("{consumer_name}_consumer")),
        deliver_policy: DeliverPolicy::All,
        filter_subject: consumer_name.to_string(),
        ack_policy: AckPolicy::Explicit,
        ack_wait: std::time::Duration::from_secs(20),
        max_deliver: 3,
        replay_policy: jetstream::consumer::ReplayPolicy::Original,
        ..Default::default()
    };

    let consumer: PullConsumer = stream
        .get_or_create_consumer(&consumer_name, config)
        .await?;

    println!("consumer: {consumer:?}");

    let mut message_stream = consumer.messages().await?;

    let mut i = 0;
    while let Some(Ok(message)) = message_stream.next().await {
        debug!(
            "got message on subject {} with payload {:?}",
            message.subject,
            from_utf8(&message.payload)?
        );
        if i % 20 == 0 {
            info!("This is a special one, sending it back");
            message.ack_with(AckKind::Nak(None)).await?;
        }
        if i % 50 == 0 {
            let pause_until = OffsetDateTime::now_utc().saturating_add(Duration::seconds_f32(10.0));
            stream.pause_consumer(&consumer_name, pause_until).await?;
            info!("stopping consumers for 10 seconds");
        }
        i += 1;
    }
    Ok(())
}
