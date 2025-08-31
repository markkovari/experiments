import { createNewBatch } from "./batchProducer.js";

try {
	const amount = Number.parseInt(process.argv[2] || "1000", 10);
	console.log({ amount });
	const run = async () => await createNewBatch(amount);

	run().then(console.log).catch(console.error);
} catch (err) {
	console.error({ err });
}
