import type { Application } from "express";
import type { Socket } from "node:net";

export function withConnectionTracking(
	app: Application,
	port: number,
) {
	const server = app.listen(port, () => {
		console.log(`Server listening on port ${port}`);
	});
	const connections = new Set<Socket>();

	server.on("connection", (socket) => {
		connections.add(socket);
		socket.on("close", () => connections.delete(socket));
	});

	const shutdown = () => {
		console.log("Shutdown initiated. Closing server, no new connection can be established and waiting for current ones to terminate");

		server.close((err) => {
			if (err) {
				console.error("Error closing server");
			}
		});
		if (connections.size === 0) {
			console.log("No connections remaining closing server");
			process.exit(0);
		}
		console.log(`Waiting for ${connections.size} active connections to drain...`);
		// Set a timeout to force exit if connections don't drain within a reasonable time
		const forceExitTimeout = setTimeout(() => {
			console.warn('Force exiting: Connections did not drain in time.');
			process.exit(1); // Exit with an error code
		}, 30 * 1000); // 30 seconds timeout for connections to drain

		const checkConnectionsInterval = setInterval(() => {
			if (connections.size === 0) {
				console.log('All active connections drained. Exiting process.');
				clearTimeout(forceExitTimeout);
				clearInterval(checkConnectionsInterval);
				process.exit(0); // Exit successfully
			} else {
				console.log(`Still waiting for ${connections.size} connections...`);
			}
		}, 500); // Check every 500ms


	};


	return {
		shutdown,
		server,
		connections,
		getActiveConnectionCount: () => connections.size,
	};
}
