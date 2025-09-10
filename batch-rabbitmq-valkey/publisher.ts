import { connect } from "amqplib";
import {
	exchangeName,
	topicName,
	getConf,
	type SomeMessage,
} from "./common.js";
import { createBatch } from "./batch.js";

const {
	mq: { url },
} = getConf();

const run = async () => {
	const connection = await connect(url);
	const channel = await connection.createChannel();
	await channel.assertExchange(exchangeName, "topic", { durable: true });

	// const randomElementLength = Math.floor(Math.random() * 4);
	const randomElementLength = 1;
	const batchElements: SomeMessage[] = [
		...new Array(randomElementLength).keys(),
	].map((i) => ({ someValue: `asd_${Math.floor(Math.random() * 1000)}` }));
	const batch = await createBatch<SomeMessage>(batchElements);
	for (const element of batch.elements) {
		const didSend = channel.publish(
			exchangeName,
			topicName,
			Buffer.from(JSON.stringify(element)),
			{ persistent: true },
		);
	}

	setTimeout(() => {
		connection.close();
		process.exit(0);
	}, 500);
};

try {
	await run();
} catch (error) {
	console.error("Cannot run publisher", { error });
}
