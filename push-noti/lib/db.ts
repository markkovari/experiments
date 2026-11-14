import { DatabaseSync, type SQLInputValue } from "node:sqlite";

// timeout: several processes (build workers, dev reloads) open the file at once.
const db = new DatabaseSync(process.env.DB_FILE ?? "data.sqlite", { timeout: 5000 });
db.exec(`
  pragma journal_mode = wal;
  create table if not exists users (id integer primary key, name text unique not null, pw text not null);
  create table if not exists sessions (token text primary key, user_id integer not null references users(id));
  create table if not exists subs (endpoint text primary key, user_id integer not null references users(id), json text not null);
  create table if not exists kv (k text primary key, v text not null);
  create table if not exists messages (
    id integer primary key, user_id integer not null references users(id), sender text not null, body text not null,
    created_at integer not null default (unixepoch()), read_at integer
  );
`);

// node:sqlite rows are untyped; callers state the row shape they selected.
export const one = <T>(sql: string, ...params: SQLInputValue[]) => db.prepare(sql).get(...params) as T | undefined;
export const all = <T>(sql: string, ...params: SQLInputValue[]) => db.prepare(sql).all(...params) as T[];
export const run = (sql: string, ...params: SQLInputValue[]) => db.prepare(sql).run(...params);
