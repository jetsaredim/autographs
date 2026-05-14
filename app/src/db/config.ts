export type DatabaseProvider = "oracle";

export type OracleDatabaseConfig = {
  provider: DatabaseProvider;
  user: string;
  password: string;
  connectString: string;
  walletDir?: string;
};

const readEnv = (name: string): string | undefined => {
  const value = process.env[name];
  return value && value.trim().length > 0 ? value : undefined;
};

export const getOracleDatabaseConfig = (): OracleDatabaseConfig => {
  const user = readEnv("ORACLE_DB_USER");
  const password = readEnv("ORACLE_DB_PASSWORD");
  const connectString = readEnv("ORACLE_DB_CONNECT_STRING");

  if (!user || !password || !connectString) {
    throw new Error(
      "Oracle database configuration is incomplete. Set ORACLE_DB_USER, ORACLE_DB_PASSWORD, and ORACLE_DB_CONNECT_STRING.",
    );
  }

  return {
    provider: "oracle",
    user,
    password,
    connectString,
    walletDir: readEnv("ORACLE_DB_WALLET_DIR"),
  };
};
