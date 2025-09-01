export const KV_BUCKET = "batch_states";
export const STREAM_NAME = "BATCH_STREAM";
export const CONSUMER_NAME = "BATCH_CONSUMER";
export const SUBJECT_PREFIX = "items.batch.";

export const serverAddress = process.env.NATS_SERVER || "localhost:4222";
