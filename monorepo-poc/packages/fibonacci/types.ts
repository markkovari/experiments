type Fibonacci = {
	cache: Map<number, number>;
	get(at: number): number;
};

export type { Fibonacci };
