const amqplib = require('amqplib');
const { getChannel, getConnection, queueName } = require("./common.js");

(async () => {
    const ch2 = await getChannel();

    setInterval(() => {
        ch2.sendToQueue(queueName, Buffer.from('something to do'));
        console.log("sent to queue")
    }, 1);
})();