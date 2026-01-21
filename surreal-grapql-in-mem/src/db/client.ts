import Surreal from 'surrealdb';

export interface DbConfig {
  url: string;
  namespace: string;
  database: string;
  username?: string;
  password?: string;
}

export async function createDbClient(config: DbConfig): Promise<Surreal> {
  const db = new Surreal();

  await db.connect(config.url);

  if (config.username && config.password) {
    await db.signin({
      username: config.username,
      password: config.password,
    });
  }

  await db.use({
    namespace: config.namespace,
    database: config.database,
  });

  return db;
}

export const defaultConfig: DbConfig = {
  url: process.env.SURREAL_URL || 'http://localhost:8000',
  namespace: process.env.SURREAL_NAMESPACE || 'test',
  database: process.env.SURREAL_DATABASE || 'test',
  username: process.env.SURREAL_USERNAME || 'root',
  password: process.env.SURREAL_PASSWORD || 'root',
};
