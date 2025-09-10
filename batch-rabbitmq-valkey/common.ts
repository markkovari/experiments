export const exchangeName = "anyExchangeName";
export const queueName = "anyQueueName";
export const topicName = "topicName";

export type RabbitMqConfig = {
	url: string;
};

export type ValKeyConf = {
	host: string;
	port: number;
	username: string;
	password: string;
};

export type Config = {
	cache: ValKeyConf;
	mq: RabbitMqConfig;
};

export const getConf = (): Config => {
	return {
		mq: {
			url: "amqp://user:password@localhost",
		},
		cache: {
			host: "localhost",
			password: "",
			port: 6379,
			username: "",
		},
	};
};

export type SomeMessage = {
	someValue: string;
};
