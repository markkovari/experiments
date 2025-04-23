const amqplib = require("amqplib");
const queueName = "tasks";


const getConnection = async () => {
    const connectionUrl = process.env.RABBIT_MQ_CONNECTION_STRING || "amqp://user:password@localhost";
    return await amqplib.connect(connectionUrl);
}

const getChannel = async () => {
    return (await getConnection()).createChannel();
}

module.exports = {
    queueName,
    getConnection,
    getChannel
}