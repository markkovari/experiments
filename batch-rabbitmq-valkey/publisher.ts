import {connect} from "amqplib";
import {  exchangeName, queueName, topicName } from "./common.js";

const run = async () => {
    const connection = await connect("amqp://user:password@localhost")
    const channel = await connection.createChannel();
    await channel.assertExchange(exchangeName, "topic", { durable: true });


    for(let i = 0; i < 2; i++) {
        const didSend = channel.publish(exchangeName, topicName, Buffer.from("SomeMessage"), {persistent: true});
    }

    setTimeout(function () {
        connection.close();
        process.exit(0);
    }, 500); 
}

try {
    await run()
} catch(error) {
    console.error("Cannot run publisher", {error})
}