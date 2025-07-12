use async_nats::jetstream;
use log::debug;
use std::{env, thread::sleep, time::Duration};

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    env_logger::init();

    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let client = async_nats::connect(nats_url).await?;

    let jetstream = jetstream::new(client);

    // Publish a few messages for the example.
    let mut i = 0;
    loop {
        let prio = match i {
            i if i % 3 == 0 => "high",
            i if i % 5 == 0 => "mid",
            _ => "low",
        };
        jetstream
            .publish(format!("events.{}", prio), "data".into())
            .await?
            .await?;
        debug!("messages sent to events.{prio}, sleeping now a bit");
        sleep(Duration::from_millis(20));
        i += 1;
    }
}
