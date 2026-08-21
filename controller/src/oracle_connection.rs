use std::env;

use oracledb::{Config, Connection};

pub fn connect(user: &str, credential: &str, connect_string: &str) -> Result<Connection, String> {
    let mut config = Config::default().set_credentials(user, credential);

    if let Some(wallet_dir) = optional_env("ORACLE_DB_WALLET_DIR") {
        let wallet_password = required_env("ORACLE_DB_WALLET_PASSWORD")?;
        config = config
            .set_config_dir(&wallet_dir)
            .set_wallet_location(&wallet_dir)
            .set_wallet_password(&wallet_password);
    }

    let config = config
        .set_connect_string(connect_string)
        .map_err(|error| format!("configure Oracle connect string: {error}"))?;
    oracledb::connect(config).map_err(|error| error.to_string())
}

fn required_env(name: &str) -> Result<String, String> {
    optional_env(name).ok_or_else(|| format!("{name} is required when ORACLE_DB_WALLET_DIR is set"))
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}
