const amqplib = require('amqplib');
const { queueName, getChannel, getConnection } = require("./common.js");

(async () => {

    const ch1 = await getChannel();
    await ch1.assertQueue(queueName);

    // Listener
    ch1.consume(queueName, (msg) => {
        if (msg !== null) {
            console.log('Received:', msg.content.toString());
            ch1.ack(msg);
        } else {
            console.log('Consumer cancelled by server');
        }
    });
})();