import express from 'express';
import winston, { createLogger } from 'winston';
import { AsyncLocalStorage } from "node:async_hooks"
import { randomUUID } from 'node:crypto';


const transport = new winston.transports.Console();
const logger = createLogger({ transports: [transport] });

const app = express();

const as = new AsyncLocalStorage();

app.use((req, res, next) => {
	const store = new Map();
	store.set("x-correlation-id", randomUUID());
	as.run(store, () => next());
});



const cLogger = () => {
	const store = as.getStore() as Map<string, string>;
	return {
		info: (message: string, meta = {}) => { logger.info(message, { ...meta, ...Object.fromEntries(store.entries()) }); },
		erro: (message: string, meta = {}) => { logger.error(message, { ...meta, ...Object.fromEntries(store.entries()) }); },
	}
};

app.get("/", (req, res) => {
	cLogger().info("req started");
	setTimeout(() => {
		const cl = cLogger().info("req ended");
		res.json({ ok: "ok" })
		return;
	}, 2000);
})


app.listen(8000, () => console.log("runs"));
