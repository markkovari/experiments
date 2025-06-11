import express, { type Request, type Response } from "express";

import { withConnectionTracking } from "./withGracefulShutdown";

const isGracefulEnabled = process.env.FEATURE_GRACEFUL_ENABLED === "true";
const port = process.env.PORT;

if (port === undefined && !Number.isInteger(port)) {
	console.error(`PORT must be a number ${port}`);
	process.exit(1);
}
const portAsNum = Number.parseInt(port as string);

const app = express();

app.get("/", (_: Request, res: Response) => {
	const randomTime = Math.floor(Math.random() * 1000);
	setTimeout(() => {
		res.send("Hello world!");
		return;
	}, randomTime);
});

const { shutdown, getActiveConnectionCount } = withConnectionTracking(
	app,
	portAsNum,
);

if (isGracefulEnabled) {
	process.on("SIGTERM", shutdown);
	process.on("SIGINT", shutdown);
	setInterval(async () => {
		console.log(`Open connections: ${getActiveConnectionCount()}`);
	}, 100);
}
