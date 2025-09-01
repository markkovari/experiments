import { sc, setupJetStreamAndKV } from "./common.js";
import { KV_BUCKET, SUBJECT_PREFIX } from "./constants.js";
import { nanos } from "nats";
import { v4 as uuidv4 } from "uuid";
import { type BatchState, BatchStatus } from "./types.js";

/**
 * @description Creates a new batch, initializes its state in the KV store, and publishes messages.
 * @param numberOfItems The number of items to include in the batch.
 */
export async function createNewBatch(numberOfItems: number): Promise<void> {
	try {
		const jsm = await setupJetStreamAndKV();
		const batchId = uuidv4();
		console.log(`[PRODUCER] Creating new batch with ID: ${batchId}`);

		const kv = await jsm.jetstream().views.kv(KV_BUCKET);
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
