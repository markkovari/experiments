import { connect } from "@nats-io/transport-node";

import { AckPolicy, jetstream, jetstreamManager, type Consumer } from "@nats-io/jetstream";
import type { Message } from "../types";
import { pauseConsumersFor } from "../halter";
import { stream, subjects } from "../constants";

const nc = await connect({ servers: ["localhost"] });

const js = jetstream(nc);
const jsm = await jetstreamManager(nc);

try {
    const highTaskConsumer = await jsm.consumers.add(stream, {
        durable_name: "task_high_consumer",
        filter_subjects: [subjects[0]!],
        ack_policy: AckPolicy.Explicit,

    });
    console.log("highTaskConsumer created", { highTaskConsumer })
} catch (e) {
    console.error("highTaskConsumer was not  created", { e })
}

const highTaskConsumer = await js.consumers.get(stream, "task_high_consumer");

try {
    const mediumTaskConsumer = await jsm.consumers.add(stream, {
        durable_name: "task_medium_consumer",
        filter_subjects: [subjects[1]!],
        ack_policy: AckPolicy.Explicit,

    });
    console.log("mediumTaskConsumer created", { mediumTaskConsumer })
} catch (e) {
    console.error("mediumTaskConsumer was not  created", { e })
}

const mediumTaskConsumer = await js.consumers.get(stream, "task_medium_consumer");


try {
    const lowTaskConsumer = await jsm.consumers.add(stream, {
        durable_name: "task_low_consumer",
        filter_subjects: [subjects[1]!],
        ack_policy: AckPolicy.Explicit,
    });
    console.log("lowTaskConsumer created", { lowTaskConsumer })
} catch (e) {
    console.error("lowTaskConsumer was not  created", { e })
}

const lowTaskConsumer = await js.consumers.get(stream, "task_low_consumer");

async function processSub(consumer: Consumer) {
    try {
        const iter = await consumer.fetch({ max_messages: 1 });
        for await (const m of iter) {
            console.info({ info: m.info })
            console.log(`Received [${m.subject}]: ${m.string()}`);
            const message = m.json<Message>();
            if (m.info.deliveryCount > 4) {
                m.term("this is a dead one rip")
                return true;
            };
            if (message.retryAfter) {
                console.info(`cannot process message, bakc to the Q and waiting ${message.retryAfter}s`)
                // pauseConsumersFor(nc, message.retryAfter * 10);
                m.nak();
                return true;
            } else {
                m.ack();
                return true;
            }
        }
    } catch {
        // likely timeout or no message
    }
    return false;
}

async function priorityLoop() {
    while (true) {
        console.log("new message")
        if (await processSub(highTaskConsumer)) continue;
        if (await processSub(mediumTaskConsumer)) continue;
        await processSub(lowTaskConsumer);
        await new Promise(r => setTimeout(r, 50));
    }
}


priorityLoop()