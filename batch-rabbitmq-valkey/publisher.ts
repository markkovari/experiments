import {connect} from "amqplib";
import {  exchangeName, topicName, getConf } from "./common.js";

const { mq: { url } } = getConf()

const run = async () => {
    const connection = await connect(url)
    const channel = await connection.createChannel();
    await channel.assertExchange(exchangeName, "topic", { durable: true });

    for(let i = 0; i < 2; i++) {
        const didSend = channel.publish(exchangeName, topicName, Buffer.from("SomeMessage"), {persistent: true});
    }

    setTimeout(() => {
        connection.close();
        process.exit(0);
    }, 500); 
}

try {
    await run()
} catch(error) {
    console.error("Cannot run publisher", {error})
}