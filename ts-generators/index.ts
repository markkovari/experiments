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

const generateUntilGenerator = generateUntil(1_000_000);

for (const curr of generateUntilGenerator) {
    console.log({ curr })
}