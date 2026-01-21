import { ApolloServer } from '@apollo/server';
import { startStandaloneServer } from '@apollo/server/standalone';
import { createDbClient, defaultConfig } from './db/client.js';
import { typeDefs } from './schema/index.js';
import { resolvers, type Context } from './resolvers/index.js';

async function main() {
  const db = await createDbClient(defaultConfig);

  const server = new ApolloServer<Context>({
    typeDefs,
    resolvers,
  });

  const { url } = await startStandaloneServer(server, {
    listen: { port: 4000 },
    context: async () => ({ db }),
  });

  console.log(`🚀 Server ready at ${url}`);
}

main().catch(console.error);
