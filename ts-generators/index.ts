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

type Stuff = {
	name: string;
	age: number;
};

const someTime = () => Math.random() * 100;

const mockedAsyncTask = (value: string): Promise<string> => {
	const time = someTime();
	return new Promise((resolve) => setTimeout(() => resolve(value), time));
};

async function* consumeStuff(
	from: number,
	to: number,
): AsyncGenerator<Stuff, void, unknown> {
	for (let i = from; i < to; i++) {
		const response = await mockedAsyncTask("thing");
		yield {
			age: i,
			name: `${response}_${i}`,
		};
	}
}

async function* consumeStuffsBatched(
	from: number,
	to: number,
	withBatch: number,
): AsyncGenerator<Stuff[]> {
	let stuffToYield: Stuff[] = [];
	for (let i = from; i < to; i++) {
		const response = await mockedAsyncTask("thing");
		stuffToYield = [
			...stuffToYield,
			{
				age: i,
				name: `${response}_${i}`,
			},
		];
		if (i % withBatch === 0) {
			yield stuffToYield;
			stuffToYield = [];
		}
	}
	if (stuffToYield.length > 0) {
		yield stuffToYield;
	}
}

const asyncGenerator = consumeStuff(1, 100);
for await (const element of asyncGenerator) {
	console.log({ element });
}
const asyncGeneratorBatched = consumeStuffsBatched(1, 100, 30);
for await (const element of asyncGeneratorBatched) {
	console.log({ element });
}
