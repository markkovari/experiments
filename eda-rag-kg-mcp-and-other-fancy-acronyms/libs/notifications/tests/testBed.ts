import { test as baseTest, expect, describe, it } from 'vitest';
import { NatsConnection } from '@nats-io/transport-node';
import { KV } from '@nats-io/kv';
import { JetStreamClient } from '@nats-io/jetstream';

interface CustomTestContext {
    natsClient: NatsConnection;
    jetStreamClient: JetStreamClient;
    natsUrl: string;
    user: string;
    pass: string;
    kvBucket: KV;
}

const test = baseTest.extend<CustomTestContext>({
    natsUrl: async ({ }, use) => {
        if (!globalThis.natsUrl) {
            throw new Error('NATS URL not initialized in global setup.');
        }
        await use(globalThis.natsUrl);
    },
    user: async ({ }, use) => {
        if (!globalThis.user) {
            throw new Error('NATS user not initialized in global setup.');
        }
        await use(globalThis.user);
    },
    pass: async ({ }, use) => {
        if (!globalThis.user) {
            throw new Error('NATS pass not initialized in global setup.');
        }
        await use(globalThis.pass);
    },
    natsClient: async ({ }, use) => {
        if (!globalThis.natsClient) {
            throw new Error('NATS client not initialized in global setup.');
        }
        await use(globalThis.natsClient);
    },
    jetStreamClient: async ({ }, use) => {
        if (!globalThis.jetStreamClient) {
            throw new Error('JetStream client not initialized in global setup.');
        }
        await use(globalThis.jetStreamClient);
    },
    kvBucket: async ({ }, use) => {
        if (!globalThis.kvBucket) {
            throw new Error('KV bucket not initialized in global setup.');
        }
        await use(globalThis.kvBucket);
    },
});

export { test, expect, describe }
export { test as it }
