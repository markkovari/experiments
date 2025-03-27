function* calculateFibonacci(): Generator<number> {
	let prev = 0;
	let curr = 1;

	yield prev;
	yield curr;

	while (true) {
		const next = prev + curr;
		yield next;
		prev = curr;
		curr = next;
	}
}

function* generateUntil(until: number): Generator<number> {
	let curr = 1;
	yield curr;
	while (curr !== until) {
		curr++;
		yield curr;
	}
}

// const fibonacciGenerator = calculateFibonacci();

// // Calculate the first 10 Fibonacci numbers
// for (let i = 0; i < 1000; i++) {
//     console.log(fibonacciGenerator.next().value);
//     // 0, 1, 1, 2, 3, 5, 8, 13, 21, 34
// }

// const generateUntilGenerator = generateUntil(1_000_000);

// for (const curr of generateUntilGenerator) {
// 	console.log({ curr });
// }

const someTime = () => (1 + Math.random()) * 1000;

const mockedAsyncTask = (value: string): Promise<string> => {
	const time = someTime();
	console.info(`Waiting for ${time}`);
	return new Promise((resolve) => setTimeout(() => resolve(value), time));
};

async function* consume(
	from: number,
	to: number,
): AsyncGenerator<string, void, unknown> {
	for (let i = from; i < to; i++) {
		const response = await mockedAsyncTask("thing");
		yield response;
	}
}

const asyncGenerator = consume(1, 100);
for await (const element of asyncGenerator) {
	console.log({ element });
}
