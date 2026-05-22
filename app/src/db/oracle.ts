import oracledb from "oracledb";
import type { Connection } from "oracledb";

import { getOracleDatabaseConfig } from "./config";

type BindValue = string | number | Date | null | undefined;

export type SqlBinds = Record<string, BindValue>;

export type QueryResult<T> = {
  rows: T[];
};

export type DatabaseExecutor = {
  execute<T>(sql: string, binds?: SqlBinds): Promise<QueryResult<T>>;
  executeMany(sql: string, binds: SqlBinds[]): Promise<void>;
  transaction<T>(work: (executor: DatabaseExecutor) => Promise<T>): Promise<T>;
};

const createExecutorFromConnection = (
  connection: Connection,
): DatabaseExecutor => ({
  async execute<T>(sql: string, binds: SqlBinds = {}): Promise<QueryResult<T>> {
    const result = await connection.execute<T>(sql, binds, {
      outFormat: oracledb.OUT_FORMAT_OBJECT,
      autoCommit: false,
      fetchInfo: {
        DESCRIPTION: { type: oracledb.STRING },
      }
    });
    return { rows: (result.rows ?? []) as T[] };
  },
  async executeMany(sql: string, binds: SqlBinds[]): Promise<void> {
    if (binds.length === 0) {
      return;
    }
    await connection.executeMany(sql, binds, { autoCommit: false });
  },
  async transaction<T>(
    work: (executor: DatabaseExecutor) => Promise<T>,
  ): Promise<T> {
    return work(createExecutorFromConnection(connection));
  },
});

export const createOracleExecutor = (): DatabaseExecutor => {
  const config = getOracleDatabaseConfig();

  const withConnection = async <T>(
    work: (connection: Connection) => Promise<T>,
  ): Promise<T> => {
    const connection = await oracledb.getConnection({
      user: config.user,
      password: config.password,
      connectString: config.connectString,
      ...(config.walletDir
        ? {
            configDir: config.walletDir,
            walletLocation: config.walletDir,
          }
        : {}),
      ...(config.walletPassword
        ? { walletPassword: config.walletPassword }
        : {}),
    });

    try {
      return await work(connection);
    } finally {
      await connection.close();
    }
  };

  return {
    async execute<T>(sql: string, binds: SqlBinds = {}): Promise<QueryResult<T>> {
      return withConnection(async (connection) => {
        const result = await connection.execute<T>(sql, binds, {
          outFormat: oracledb.OUT_FORMAT_OBJECT,
          autoCommit: true,
          fetchInfo: {
            DESCRIPTION: { type: oracledb.STRING },
          }
        });
        return { rows: (result.rows ?? []) as T[] };
      });
    },
    async executeMany(sql: string, binds: SqlBinds[]): Promise<void> {
      if (binds.length === 0) {
        return;
      }
      await withConnection(async (connection) => {
        await connection.executeMany(sql, binds, { autoCommit: true });
      });
    },
    async transaction<T>(
      work: (executor: DatabaseExecutor) => Promise<T>,
    ): Promise<T> {
      return withConnection(async (connection) => {
        const executor = createExecutorFromConnection(connection);
        try {
          const result = await work(executor);
          await connection.commit();
          return result;
        } catch (error) {
          await connection.rollback();
          throw error;
        }
      });
    },
  };
};
