import type { NatsConnection } from "@nats-io/transport-node";

import { jetstreamManager } from "@nats-io/jetstream";
import { stream } from "../constants";


const pauseConsumersFor = async (nc: NatsConnection, forMs = 20 * 1000) => {
    const jsm = await jetstreamManager(nc);
    const now = new Date();
    const aMinuteAfter = now.getTime() + forMs;
    const aMinuteAfterInDate = new Date(aMinuteAfter);
    for await (const c of jsm.consumers.list(stream)) {
        await jsm.consumers.pause(stream, c.name, aMinuteAfterInDate)
    }
}

export {
    pauseConsumersFor
}