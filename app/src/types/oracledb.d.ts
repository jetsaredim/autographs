declare module "oracledb" {
  export const OUT_FORMAT_OBJECT: number;

  export type BindParameters = Record<string, unknown>;

  export type ExecuteOptions = {
    outFormat?: number;
    autoCommit?: boolean;
  };

  export type ExecuteManyOptions = {
    autoCommit?: boolean;
  };

  export type Result<T> = {
    rows?: T[];
  };

  export type Connection = {
    execute<T>(
      sql: string,
      binds?: BindParameters,
      options?: ExecuteOptions,
    ): Promise<Result<T>>;
    executeMany(
      sql: string,
      binds: BindParameters[],
      options?: ExecuteManyOptions,
    ): Promise<void>;
    commit(): Promise<void>;
    rollback(): Promise<void>;
    close(): Promise<void>;
  };

  export type ConnectionAttributes = {
    user: string;
    password: string;
    connectString: string;
  };

  export function getConnection(
    attributes: ConnectionAttributes,
  ): Promise<Connection>;

  const oracledb: {
    OUT_FORMAT_OBJECT: typeof OUT_FORMAT_OBJECT;
    getConnection: typeof getConnection;
  };

  export default oracledb;
}
