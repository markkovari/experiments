import { randomBytes } from "node:crypto";
import { describe, expect, test as baseTest } from "vitest";
import { userNotificationsBucketName } from "../notifications";
import type { StartedTestContainer } from "testcontainers";

import { Kvm, type KV } from "@nats-io/kv";
import { connect, type NatsConnection } from "@nats-io/transport-node";
import { NatsContainer } from "@testcontainers/nats";

export interface CustomTestContext {
	context: {
		natsAccess: {
			url: string;
			pass: string;
			user: string;
		};
		connection: NatsConnection;
		container: StartedTestContainer;
		bucket: KV;
		close: () => Promise<void>;
	};
}

const randomString = (length: number = 16) => {
	return Buffer.from(randomBytes(length)).toString("hex");
};

export async function setupNotifications(): Promise<CustomTestContext> {
	const user = randomString();
	const pass = randomString();
	const container = await new NatsContainer("nats:latest")
		.withJetStream()
		.withUsername(user)
		.withPass(pass)
		.start();
	const url = `nats://${container.getHost()}:${container.getMappedPort(4222)}`;
	const connection = await connect({ servers: [url], user, pass });

	const keyValueManager = new Kvm(connection);

	const bucket = await keyValueManager.create(userNotificationsBucketName);

	return {
		context: {
			natsAccess: {
				url,
				pass,
				user,
			},
			container,
			connection,
			bucket,
			close: async () => {
				await connection.drain();
				await connection.close();
				await container.stop();
			},
		},
	};
}

const test = baseTest.extend<CustomTestContext>({
	context: async ({ onTestFailed, onTestFinished }, use) => {
		const context = await setupNotifications();
		await use(context.context);
		onTestFailed(async () => {
			// await context.context.close();
		});
		onTestFinished(async () => {
			// await context.context.close();
		});
	},
});

export { test, expect, describe };
export { test as it };
