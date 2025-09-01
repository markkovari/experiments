import { connect, type KvEntry, AckPolicy } from "nats";
import { BatchStatus, type BatchState } from "./types.js";
import {
	CONSUMER_NAME,
	KV_BUCKET,
	serverAddress,
	STREAM_NAME,
	SUBJECT_PREFIX,
} from "./constants.js";
import { sc } from "./common.js";

/**
 * @description Runs the NATS consumer to process batch items and update state.
 */
export async function runConsumer(): Promise<void> {
	try {
		const nc = await connect({ servers: serverAddress });
		const jsm = await nc.jetstreamManager();
		const js = nc.jetstream();
		const consumerName = `${CONSUMER_NAME}_${Math.floor(Math.random() * 100000)}`;

		await jsm.consumers.add(STREAM_NAME, {
			durable_name: consumerName,
			ack_policy: AckPolicy.Explicit,
			filter_subject: `${SUBJECT_PREFIX}>`,
		});

		const kv = await jsm.jetstream().views.kv(KV_BUCKET);
		const consumer = await js.consumers.get(STREAM_NAME, consumerName);
		const sub = await consumer.consume();
		console.log("[CONSUMER] Consumer is listening for messages...");

		for await (const message of sub) {
			const payload = JSON.parse(sc.decode(message.data));
			const { batchId, itemId } = payload;

			console.log(`[CONSUMER] Processing item ${itemId} from batch ${batchId}`);

			try {
				const kvEntry = (await kv.get(batchId)) as KvEntry | null;
				if (!kvEntry) {
					throw new Error(`State for batch ${batchId} not found.`);
				}

				const currentState: BatchState = JSON.parse(sc.decode(kvEntry.value));
				currentState.completedItems++;

				await kv.put(batchId, sc.encode(JSON.stringify(currentState)));

				message.ack();

				if (currentState.completedItems === currentState.totalItems) {
					console.log(`[CONSUMER] Batch ${batchId} is now complete.`);
					if (currentState.failedItems.length > 0) {
						console.log(
							`[CONSUMER] Batch ${batchId} failed with the following items: ${currentState.failedItems.join(", ")}`,
						);
						currentState.status = BatchStatus.PartialSuccess;
					} else {
						console.log(
							`[CONSUMER] Batch ${batchId} was successful! All items processed.`,
						);
						currentState.status = BatchStatus.Completed;
					}
				}
				await kv.put(batchId, sc.encode(JSON.stringify(currentState)));
			} catch (error: any) {
				console.error(
					`[CONSUMER] Failed to process item ${itemId}:`,
					error.message,
					error,
				);
				try {
					const kvEntry = (await kv.get(batchId)) as KvEntry | null;
					if (kvEntry) {
						const currentState: BatchState = JSON.parse(
							sc.decode(kvEntry.value),
						);
						currentState.failedItems.push(itemId);
						await kv.put(batchId, sc.encode(JSON.stringify(currentState)));
					}
				} catch (updateError) {
					console.error(
						"[CONSUMER] Failed to update KV state for failed item:",
						updateError,
					);
				}
				message.ack();
			}
		}
	} catch (error) {
		console.error("[CONSUMER] Failed to run consumer:", error);
	}
}
