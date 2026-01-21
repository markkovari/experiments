import type Surreal from 'surrealdb';
import { RecordId } from 'surrealdb';

interface DbUser {
  id: RecordId;
  name: string;
  email: string;
  [key: string]: unknown;
}

interface CreateUserInput {
  name: string;
  email: string;
}

interface UpdateUserInput {
  name?: string;
  email?: string;
}

export interface Context {
  db: Surreal;
}

function formatUser(user: DbUser): { id: string; name: string; email: string } {
  return {
    id: user.id.toString(),
    name: user.name,
    email: user.email,
  };
}

function parseRecordId(id: string): RecordId | null {
  const match = id.match(/^(\w+):(.+)$/);
  if (!match) return null;
  return new RecordId(match[1], match[2]);
}

export const resolvers = {
  Query: {
    users: async (_: unknown, __: unknown, { db }: Context) => {
      const users = await db.select<DbUser>('user');
      return (users as DbUser[]).map(formatUser);
    },

    user: async (_: unknown, { id }: { id: string }, { db }: Context) => {
      const recordId = parseRecordId(id);
      if (!recordId) return null;

      try {
        const user = await db.select<DbUser>(recordId);
        return user ? formatUser(user as DbUser) : null;
      } catch {
        return null;
      }
    },
  },

  Mutation: {
    createUser: async (
      _: unknown,
      { input }: { input: CreateUserInput },
      { db }: Context
    ) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const result = await (db as any).create('user', {
        name: input.name,
        email: input.email,
      });
      const users = result as DbUser[];
      return formatUser(users[0]);
    },

    updateUser: async (
      _: unknown,
      { id, input }: { id: string; input: UpdateUserInput },
      { db }: Context
    ) => {
      const recordId = parseRecordId(id);
      if (!recordId) return null;

      try {
        const user = await db.merge<DbUser>(recordId, {
          ...(input.name !== undefined && { name: input.name }),
          ...(input.email !== undefined && { email: input.email }),
        });
        return user ? formatUser(user as DbUser) : null;
      } catch {
        return null;
      }
    },

    deleteUser: async (_: unknown, { id }: { id: string }, { db }: Context) => {
      const recordId = parseRecordId(id);
      if (!recordId) return false;

      try {
        const user = await db.delete<DbUser>(recordId);
        return user !== null && user !== undefined;
      } catch {
        return false;
      }
    },
  },
};
