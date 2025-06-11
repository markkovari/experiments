import express from "express";
import type { Request, Response, NextFunction } from "express";

const app = express();

function requestLogger(req: Request, res: Response, next: NextFunction) {
	// Record the start time of the request
	const startTime = process.hrtime();

	// Log the start of the request
	console.log(
		`[${new Date().toISOString()}] Incoming Request: ${req.method} ${req.originalUrl}`
	);

	// Set up an event listener for when the response finishes
	res.on('finish', () => {
		// Calculate the elapsed time since the request started
		const [seconds, nanoseconds] = process.hrtime(startTime);
		const durationMs = (seconds * 1000) + (nanoseconds / 1000000);

		// Log the end of the request, including the status code and duration
		console.log(
			`[${new Date().toISOString()}] Finished Request: ${req.method} ${req.originalUrl} - Status: ${res.statusCode} - Duration: ${durationMs.toFixed(2)}ms`
		);
	});

	// Pass control to the next middleware or route handler
	next();
}

app.use(requestLogger);

app.get("/", (req, res) => {
	const randomWaitTime = 20000;
	setTimeout(() => {

		res.status(200).json({ asd: 2 })
		return
	}, randomWaitTime)

})


const server = app.listen(8000, () => console.log("Server is running on port 8000"))

process.on('SIGTERM', () => {
	console.log('SIGTERM signal received: closing HTTP server')
	server.close(() => {
		console.log('HTTP server closed')
	})
})

