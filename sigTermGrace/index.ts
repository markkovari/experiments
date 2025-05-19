import express, { type Request, type Response } from 'express'

import { withConnectionTracking } from "./withGracefulShutdown"

const isGracefulEnabled = process.env.FEATURE_GRACEFUL_ENABLED === "true";

const app = express();

app.get('/', (req: Request, res: Response) => {
    res.send('Hello world!');
    return;
});

const { shutdown, getActiveConnectionCount } = withConnectionTracking(app, 8000, isGracefulEnabled);

if (isGracefulEnabled) {
    process.on('SIGTERM', shutdown);
    process.on('SIGINT', shutdown);
    setInterval(() => {
        console.log(`Open connections: ${getActiveConnectionCount()}`);
    }, 100);
}
