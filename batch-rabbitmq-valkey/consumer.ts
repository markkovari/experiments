import { type ConsumeMessage, connect } from "amqplib";
import { exchangeName, queueName, topicName } from "./common.js";

const run = async () => {
	const connection = await connect("amqp://user:password@localhost");
	const channel = await connection.createChannel();
	await channel.assertExchange(exchangeName, "topic", { durable: true });
	await channel.bindQueue(queueName, exchangeName, topicName);

	await channel.consume(queueName, (msg: ConsumeMessage | null) => {
		if (msg) {
			console.log({ msg });
		}
	});
};

try {
	await run();
} catch (error) {
	console.error("Cannot run consumer", { error });
}
