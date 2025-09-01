import { createNewBatch } from "./batchProducer.js";

try {
	const amount = Number.parseInt(process.argv[2] || "1000", 10);
	const parallel = Number.parseInt(process.argv[3] || "10", 10);

	console.log({ parallel, amount });
	const response = await Promise.all(
		[...new Array(parallel).keys()].map((_) => createNewBatch(amount)),
	);
	console.log({ response });
} catch (err) {
	console.error({ err });
}
