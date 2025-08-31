import { connect, StringCodec, nanos, KvEntry, AckPolicy } from "nats";
import type { BatchState } from "./types.js";

const sc = StringCodec();
const KV_BUCKET = "batch_states";
const STREAM_NAME = "BATCH_STREAM";
const CONSUMER_NAME = "BATCH_CONSUMER";

/**
 * @description Runs the NATS consumer to process batch items and update state.
 */
export async function runConsumer(): Promise<void> {
	try {
		const nc = await connect({ servers: "localhost:4222" });
		const jsm = await nc.jetstreamManager();
		const js = nc.jetstream();

		// Ensure the consumer exists for the stream.
		await jsm.consumers.add(STREAM_NAME, {
			durable_name: CONSUMER_NAME,
			ack_policy: AckPolicy.Explicit,
			filter_subject: `items.batch.>`,
		});

		const kv = await jsm.jetstream(STREAM_NAME).get(KV_BUCKET);
		const consumer = await js.consumers.get(STREAM_NAME, CONSUMER_NAME);

		console.log("[CONSUMER] Consumer is listening for messages...");

		for await (const message of consumer.consume()) {
			const payload = JSON.parse(sc.decode(message.data));
			const { batchId, itemId } = payload;

			console.log(`[CONSUMER] Processing item ${itemId} from batch ${batchId}`);

			try {
				// Simulate processing logic, with a 10% chance of failure for demonstration.
				if (Math.random() < 0.1) {
					throw new Error("Simulated processing failure");
				}

				// Get the latest state from the KV bucket.
				const kvEntry = (await kv.get(batchId)) as KvEntry | null;
				if (!kvEntry) {
					throw new Error(`State for batch ${batchId} not found.`);
				}

				const currentState: BatchState = JSON.parse(sc.decode(kvEntry.value));
				currentState.completedItems++;

				// Update the batch state in the KV bucket. The `cas` (Compare And Swap) ensures
				// we only update if the revision number hasn't changed, preventing race conditions.
				await kv.update(
					batchId,
					sc.encode(JSON.stringify(currentState)),
					kvEntry.revision,
				);

				// Acknowledge the message to signal successful processing.
				message.ack();

				// Check for batch completion.
				if (currentState.completedItems === currentState.totalItems) {
					console.log(`[CONSUMER] Batch ${batchId} is now complete.`);
					if (currentState.failedItems.length > 0) {
						console.log(
							`[CONSUMER] Batch ${batchId} failed with the following items: ${currentState.failedItems.join(", ")}`,
						);
					} else {
						console.log(
							`[CONSUMER] Batch ${batchId} was successful! All items processed.`,
						);
					}
				}
			} catch (error: any) {
				console.error(
					`[CONSUMER] Failed to process item ${itemId}:`,
					error.message,
				);

				// If processing fails, update the KV state to record the failure.
				// We will attempt to update the state regardless of optimistic locking failure
				// because the next message update will eventually handle it.
				try {
					const kvEntry = (await kv.get(batchId)) as KvEntry | null;
					if (kvEntry) {
						const currentState: BatchState = JSON.parse(
							sc.decode(kvEntry.value),
						);
						currentState.failedItems.push(itemId);
						await kv.update(
							batchId,
							sc.encode(JSON.stringify(currentState)),
							kvEntry.revision,
						);
					}
				} catch (updateError) {
					console.error(
						"[CONSUMER] Failed to update KV state for failed item:",
						updateError,
					);
				}

				// NAK the message to re-queue it for another attempt (if consumer allows).
				// message.nak();
				// For this example, we will just ack it after a failure to move on.
				message.ack();
			}
		}
	} catch (error) {
		console.error("[CONSUMER] Failed to run consumer:", error);
	}
}

// Example usage:
// To run this file, call `node consumer.js` or `ts-node consumer.ts`.
// runConsumer();
