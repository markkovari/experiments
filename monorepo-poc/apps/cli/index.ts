import { calc } from "fibonacci";
import { z } from "zod";

const fibonacciParamSchema = z
	.number({ message: "value should be number" })
	.positive({
		message: "value should be number",
	});

const param = process.argv[2];

const validated = fibonacciParamSchema.safeParse(+param);

if (!validated.success) {
	console.log({ param, validated, err: validated.error });
	process.stdout.write(`Fibonacci is not calculatable ${param}`);
	process.exit(1);
}
const result = calc(validated.data);
process.stdout.write(`The result is ${result}`);
process.exit(0);
