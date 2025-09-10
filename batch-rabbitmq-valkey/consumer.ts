import { type ConsumeMessage, connect } from "amqplib";
import {
	exchangeName,
	queueName,
	topicName,
	type SomeMessage,
} from "./common.js";
import {
	updateBatchWithJobStatus,
	type Envelope,
	type Status,
} from "./batch.js";

const run = async () => {
	const connection = await connect("amqp://user:password@localhost");
	const channel = await connection.createChannel();
	await channel.assertExchange(exchangeName, "topic", { durable: true });
	await channel.bindQueue(queueName, exchangeName, topicName);

	await channel.consume(
		queueName,
		async (msg: ConsumeMessage | null) => {
			if (msg) {
				try {
					console.log({
						cont: msg.content.toString(),
						p: JSON.parse(msg.content.toString()),
					});
					const value: Envelope<SomeMessage> = JSON.parse(
						msg.content.toString(),
					);
					const outcome: Status = Math.random() > 0.05 ? "success" : "error";
					console.log("updating element", { id: value.id, status: outcome });
					await updateBatchWithJobStatus(value.batchID, value.id, outcome);
					channel.ack(msg);
				} catch (e) {
					console.error("error with message", { e });
					channel.nack(msg);
				}
			}
		},
		{ noAck: false },
	);
};

try {
	await run();
} catch (error) {
	console.error("Cannot run consumer", { error });
}
