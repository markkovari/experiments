import {
	connect,
	StorageType,
	StoreCompression,
	StringCodec,
	type JetStreamManager,
} from "nats";
import {
	KV_BUCKET,
	serverAddress,
	STREAM_NAME,
	SUBJECT_PREFIX,
} from "./constants.js";

export const sc = StringCodec();

/**
 * @description Initializes NATS connection and sets up JetStream and KV bucket.
 * @returns A promise that resolves with the JetStream manager instance.
 */
export async function setupJetStreamAndKV(): Promise<JetStreamManager> {
	const nc = await connect({ servers: serverAddress });
	const jsm = await nc.jetstreamManager();
	// Add a stream for batch items. The subject uses a wildcard to capture all items.
	await jsm.streams.add({
		name: STREAM_NAME,
		storage: StorageType.File,
		compression: StoreCompression.S2,
		description: "Batch stream to send things through",
		subjects: [`${SUBJECT_PREFIX}>`],
		max_age: 5 * 60 * 1000 * 1000 * 1000, // 5 minutes
	});
	await jsm.jetstream().views.kv(KV_BUCKET, {
		ttl: 2 * 60 * 1000 * 1000 * 1000, // 2 minutes
		compression: true,
		backingStore: StorageType.Memory,
		description: "Batch storage KV",
	});

	return jsm;
}
