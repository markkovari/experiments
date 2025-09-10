import { GlideClient, TimeUnit } from "@valkey/valkey-glide";
import { getConf } from "./common.js";

/**
 * A generic Redis-backed cache implementation using Valkey's Glide client.
 * Provides methods to interact with the cache, including set, get, delete, and clear operations.
 */
class RedisCache<T = unknown> {
	/**
	 * Static instance of the GlideClient used for Redis communication.
	 * Shared across all instances of RedisCache.
	 */
	static client: GlideClient;

	/**
	 * Factory method to create a new typed RedisCache instance.
	 * Ensures the Redis client is initialized before returning a type-safe cache.
	 *
	 * The generic parameter `<T>` allows consumers to define the expected shape
	 * of values stored in or retrieved from the cache.
	 */
	static async factory<T>() {
		await RedisCache.getClient();
		return new RedisCache<T>();
	}

	/**
	 * Lazily initializes and returns the Redis Glide client.
	 * Pulls connection details from centralized configuration.
	 */
	static async getClient() {
		if (RedisCache.client) {
			return RedisCache.client;
		}
		const {
			cache: { host, port, username, password },
		} = getConf();
		const client = await GlideClient.createClient({
			addresses: [{ host, port }],
			credentials: {
				password,
				username,
			},
		});
		RedisCache.client = client;
		return client;
	}

	/**
	 * Default time-to-live (TTL) for cached entries, in milliseconds.
	 * Defaults to 5 minutes.
	 */
	private DEFAULT_TTL_MS = 5 * 60 * 1000;

	/**
	 * Stores a key-value pair in the Redis cache.
	 * Accepts an optional TTL to specify expiry.
	 * Serializes non-string values to JSON.
	 */
	async set(key: string, value: T, TTL = this.DEFAULT_TTL_MS) {
		const toStore = typeof value === "string" ? value : JSON.stringify(value);

		return await (await RedisCache.getClient()).set(key, toStore, {
			expiry: TTL
				? { count: TTL, type: TimeUnit.Milliseconds }
				: "keepExisting",
		});
	}

	/**
	 * Retrieves a value from the cache by its key.
	 * Returns null if the key does not exist.
	 */
	async get(key: string): Promise<T | null> {
		return (await (await RedisCache.getClient()).get(key)) as T;
	}

	/**
	 * Deletes a specific key from the Redis cache.
	 */
	async delete(key: string) {
		await (await RedisCache.getClient()).del([key]);
	}

	/**
	 * Clears all entries from the current Redis database.
	 * Use with caution — this flushes the entire cache.
	 */
	async clear() {
		await (await RedisCache.getClient()).flushdb();
	}
}

export { RedisCache };
