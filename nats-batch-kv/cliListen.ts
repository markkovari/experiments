import { runConsumer } from "./batchConsumer.js";

runConsumer().then(console.log).catch(console.error);
