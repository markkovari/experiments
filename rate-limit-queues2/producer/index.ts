import { connect } from "@nats-io/transport-node";

import { jetstreamManager } from "@nats-io/jetstream";
import { stream, subjects } from "../constants";
import type { Message } from "../types";

const nc = await connect({ servers: ["localhost"] });

const jsm = await jetstreamManager(nc);

await jsm.streams.add({ name: stream, subjects: subjects });


setInterval(() => {

    const hasRetryAfter = Math.random() < 0.05;
    const retryAfterSeconds = Math.floor(Math.random() * 5)

    const msg: Message = { message: "hello" }
    if (hasRetryAfter) {
        msg.retryAfter = retryAfterSeconds;
    }

    const asString = JSON.stringify(msg)
    const rand = Math.random();
    if (rand < 0.03) {
        console.log("to high")
        nc.publish(subjects[0]!, asString);
    } else if (rand < 0.3) {
        console.log("to medium")
        nc.publish(subjects[1]!, asString);
    } else {
        console.log("to low")
        nc.publish(subjects[2]!, asString);
    }

    console.log("sent", new Date())
    console.groupEnd()
}, 10);
