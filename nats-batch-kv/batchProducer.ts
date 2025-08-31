import { connect, nanos, StringCodec, JetStreamManager } from "nats";
import { v4 as uuidv4 } from "uuid";
import { BatchState, BatchStatus } from "./types";

const sc = StringCodec();
const SUBJECT_PREFIX = "items.batch.";
const KV_BUCKET = "batch_states";

/**
 * @description Initializes NATS connection and sets up JetStream and KV bucket.
 * @returns A promise that resolves with the JetStream manager instance.
 */
async function setupJetStreamAndKV(): Promise<JetStreamManager> {
	const nc = await connect({ servers: "localhost:4222" });
	const jsm = await nc.jetstreamManager();

	// Add a stream for batch items. The subject uses a wildcard to capture all items.
	await jsm.streams.add({
		name: "BATCH_STREAM",
		subjects: [`${SUBJECT_PREFIX}>`],
	});

	// Add a key-value bucket to track batch states.
	await jsm.kv.add({ bucket: KV_BUCKET });

	return jsm;
}

/**
 * @description Creates a new batch, initializes its state in the KV store, and publishes messages.
 * @param numberOfItems The number of items to include in the batch.
 */
export async function createNewBatch(numberOfItems: number): Promise<void> {
	try {
		const jsm = await setupJetStreamAndKV();
		const batchId = uuidv4();
		console.log(`[PRODUCER] Creating new batch with ID: ${batchId}`);

		const kv = await jsm.kv.get(KV_BUCKET);
		const batchState: BatchState = {
			status: BatchStatus.Pending,
			totalItems: numberOfItems,
			completedItems: 0,
			failedItems: [],
			createdAt: nanos(Date.now()),
		};

		// Initialize the batch state in the KV bucket.
		await kv.put(batchId, sc.encode(JSON.stringify(batchState)));
		console.log(
			`[PRODUCER] Initialized state for batch ${batchId} in KV bucket.`,
		);

		// Publish each item to the NATS stream.
		const js = jsm.jetstream();
		for (let i = 0; i < numberOfItems; i++) {
			const itemId = `${batchId}-${i}`;
			const payload = {
				batchId,
				itemId,
				// Add any item-specific data here
				data: `Item data for item ${itemId}`,
			};
			const subject = `${SUBJECT_PREFIX}${batchId}`;
			await js.publish(subject, sc.encode(JSON.stringify(payload)));
			console.log(`[PRODUCER] Published item ${itemId} for batch ${batchId}`);
		}

		console.log(
			`[PRODUCER] All items for batch ${batchId} have been published.`,
		);
		console.log(`[PRODUCER] Batch creation complete.`);
	} catch (error) {
		console.error("[PRODUCER] Failed to create batch:", error);
	}
}

// Example usage:
// To run this file, call `node producer.js` or `ts-node producer.ts`.
// createNewBatch(10);
