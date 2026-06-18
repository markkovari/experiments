// Shared host shim for `wasi:keyvalue/store@0.2.0-draft` + `/atomics`.
//
// Every transpiled component in this example (the composed auth-guard, plus
// session-store, search-index, validate, config-store, notify-dispatch) maps
// its keyvalue import to THIS module, so they all read/write one in-memory
// store. That's the whole trick that lets the components cooperate: a pet the
// app writes under `pet_*` is indexed by search-index and sits beside the
// sessions auth-guard mints and the audit events it records — one process, one
// Map, no network.
//
// Swap the Map for redis/sqlite/NATS to make it durable; the components don't
// care. (Same shim as examples/jco-embed; copied verbatim.)

const buckets = new Map(); // name -> Map<string, Uint8Array>

class Bucket {
  constructor(name) {
    if (!buckets.has(name)) buckets.set(name, new Map());
    this.store = buckets.get(name);
  }
  get(key) {
    return this.store.get(key); // Uint8Array | undefined  (option<list<u8>>)
  }
  set(key, value) {
    this.store.set(key, value);
  }
  delete(key) {
    this.store.delete(key);
  }
  exists(key) {
    return this.store.has(key);
  }
  listKeys(_cursor) {
    return { keys: [...this.store.keys()], cursor: undefined };
  }
}

export { Bucket };
export function open(name) {
  return new Bucket(name);
}

// wasi:keyvalue/atomics — the rate-limiter (composed into auth-guard) and the
// quota-style counters bump values atomically. Single Node process => naturally
// atomic. Counter stored as a UTF-8 decimal string.
export function increment(bucket, key, delta) {
  const cur = bucket.get(key);
  const n = (cur ? Number(new TextDecoder().decode(cur)) : 0) + Number(delta);
  bucket.set(key, new TextEncoder().encode(String(n)));
  return BigInt(n);
}

// Test/dev helper: wipe the whole store between test runs.
export function __reset() {
  buckets.clear();
}
