import type { Application } from "express";
import type { Socket } from "node:net";

export function withConnectionTracking(
	app: Application,
	port: number,
	isEnabled: boolean,
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
		console.log("Shutdown initiated. Closing server to new connections...");

		server.close(() => {
			console.log("HTTP server closed. All requests finished.");
			process.exit(0);
		});

		setTimeout(() => {
			console.warn("Force shutdown: destroying open sockets...");
			for (const socket of connections) {
				socket.destroy();
			}
			process.exit(1);
		}, 10000);
	};

	return {
		shutdown,
		server,
		getActiveConnectionCount: () => connections.size,
	};
}
