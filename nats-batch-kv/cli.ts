import { createNewBatch } from "./batchProducer.js";

const run = async () => await createNewBatch(1000);

run().then(console.log).catch(console.error);
