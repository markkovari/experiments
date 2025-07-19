import { connect } from "@nats-io/transport-node"
import { Kvm } from "@nats-io/kv";

const nc = connect(["localhost:4222"])

// using a nats connection:
const kvm = new Kvm(nc);
await kvm.list();
await kvm.create("mykv");
