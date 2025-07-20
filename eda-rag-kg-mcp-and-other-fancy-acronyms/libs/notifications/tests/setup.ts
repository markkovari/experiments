import { StartedTestContainer } from 'testcontainers';
import { NatsContainer } from '@testcontainers/nats';
import { connect, NatsConnection, } from '@nats-io/transport-node';
import { Kvm, type KV } from '@nats-io/kv'
import { JetStreamClient, jetstreamManager } from '@nats-io/jetstream'
import { afterAll, beforeAll } from 'vitest';
import { userNotificationsBucketName } from '../notifications';

declare global {
    var natsContainer: StartedTestContainer;
    var natsUrl: string;
    var pass: string;
    var user: string;
    var natsClient: NatsConnection;
    var jetStreamClient: JetStreamClient;
    var kvBucket: KV;
}


export async function setup() {
    const user = "SomeUser";
    const pass = "SomePassword";
    globalThis.natsContainer = await new NatsContainer("nats:latest").withJetStream().withUsername(user).withPass(pass).start();
    const natsUrl = `nats://${globalThis.natsContainer.getHost()}:${globalThis.natsContainer.getMappedPort(4222)}`;
    global.natsUrl = natsUrl;
    global.user = user;
    global.pass = pass;
    globalThis.natsClient = await connect({ servers: [natsUrl], user, pass });
    const jetstreamManagerInstance = await jetstreamManager(globalThis.natsClient);

    globalThis.jetStreamClient = jetstreamManagerInstance.jetstream();
    const keyValueManager = new Kvm(globalThis.natsClient);

    globalThis.kvBucket = await keyValueManager.create(userNotificationsBucketName);
}

export async function teardown() {
    if (globalThis.natsClient) {
        await globalThis.natsClient.drain();
        await globalThis.natsClient.close();
    }
    if (globalThis.natsContainer) {
        await globalThis.natsContainer.stop();
    }
}


beforeAll(async () => {
    await setup()
})

afterAll(async () => {
    await teardown()
})