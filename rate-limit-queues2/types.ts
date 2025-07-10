import type { Payload } from "@nats-io/transport-node";

export type Envelope = Payload;

export type Message = {
    message: string;
    retryAfter?: number
}