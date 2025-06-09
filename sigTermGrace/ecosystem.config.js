module.exports = {
	apps: [
		{
			name: "App 1",
			script: "PORT=8081 pnpm run start",
		},
		{
			name: "App 1",
			script: "PORT=8082 pnpm run start",
		},
	],
};
