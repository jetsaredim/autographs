import { promises as fs } from "node:fs";
import path from "node:path";

import type { DatabaseExecutor } from "./oracle";

type MigrationRow = {
  VERSION: string;
};

export type AppliedMigration = {
  version: string;
  name: string;
};

const migrationsDir = path.join(process.cwd(), "db", "migrations");

const ensureMigrationsTable = async (executor: DatabaseExecutor): Promise<void> => {
  try {
    await executor.execute(`
      create table autograph_schema_migrations (
        version varchar2(32) primary key,
        name varchar2(255) not null,
        applied_at timestamp default current_timestamp not null
      )
    `);
  } catch (error) {
    if (!isAlreadyExistsError(error)) {
      throw error;
    }
  }
};

const isAlreadyExistsError = (error: unknown): boolean =>
  error instanceof Error && /ORA-00955/.test(error.message);

const splitStatements = (sql: string): string[] =>
  sql
    .split(/\n\s*\n/g)
    .map((statement) => statement.trim().replace(/;$/, ""))
    .filter(Boolean);

export const runMigrations = async (
  executor: DatabaseExecutor,
): Promise<AppliedMigration[]> => {
  await ensureMigrationsTable(executor);

  const existing = await executor.execute<MigrationRow>(
    "select version from autograph_schema_migrations",
  );
  const appliedVersions = new Set(existing.rows.map((row) => row.VERSION));
  const files = (await fs.readdir(migrationsDir))
    .filter((file) => /^\d+_.+\.sql$/.test(file))
    .sort();
  const applied: AppliedMigration[] = [];

  for (const file of files) {
    const [version] = file.split("_");
    if (!version || appliedVersions.has(version)) {
      continue;
    }

    const sql = await fs.readFile(path.join(migrationsDir, file), "utf8");
    const name = file.replace(/^\d+_/, "").replace(/\.sql$/, "");

    await executor.transaction(async (tx) => {
      for (const statement of splitStatements(sql)) {
        try {
          await tx.execute(statement);
        } catch (error) {
          if (!isAlreadyExistsError(error)) {
            throw error;
          }
        }
      }
      await tx.execute(
        `insert into autograph_schema_migrations (version, name)
         values (:version, :name)`,
        { version, name },
      );
    });

    applied.push({ version, name });
  }

  return applied;
};
