use std::{env, time::Duration};

use oracle::Connection;
use tokio::time::{self, MissedTickBehavior};

const HEARTBEAT_INTERVAL_ENV: &str = "AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS";
const DEFAULT_HEARTBEAT_INTERVAL_SECONDS: u64 = 24 * 60 * 60;

pub fn spawn(user: String, credential: String, connect_string: String) -> Result<(), String> {
    let Some(interval) = heartbeat_interval_from_env()? else {
        tracing::info!("Oracle catalog heartbeat disabled");
        return Ok(());
    };

    tracing::info!(
        %user,
        %connect_string,
        interval_seconds = interval.as_secs(),
        "starting Oracle catalog heartbeat"
    );

    tokio::spawn(async move {
        let first_tick = time::Instant::now() + interval;
        let mut ticker = time::interval_at(first_tick, interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;

            let result = tokio::task::spawn_blocking({
                let user = user.clone();
                let credential = credential.clone();
                let connect_string = connect_string.clone();
                move || run_heartbeat(&user, &credential, &connect_string)
            })
            .await;

            match result {
                Ok(Ok(())) => tracing::info!("Oracle catalog heartbeat succeeded"),
                Ok(Err(error)) => {
                    tracing::warn!(%error, "Oracle catalog heartbeat failed");
                }
                Err(error) => {
                    tracing::warn!(%error, "Oracle catalog heartbeat task failed");
                }
            }
        }
    });

    Ok(())
}

fn heartbeat_interval_from_env() -> Result<Option<Duration>, String> {
    match env::var(HEARTBEAT_INTERVAL_ENV) {
        Ok(value) => parse_heartbeat_interval(Some(value.as_str())),
        Err(env::VarError::NotPresent) => parse_heartbeat_interval(None),
        Err(env::VarError::NotUnicode(_)) => {
            Err(format!("{HEARTBEAT_INTERVAL_ENV} must be valid UTF-8"))
        }
    }
}

fn parse_heartbeat_interval(raw: Option<&str>) -> Result<Option<Duration>, String> {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(Some(Duration::from_secs(
            DEFAULT_HEARTBEAT_INTERVAL_SECONDS,
        )));
    };

    let seconds = raw.parse::<u64>().map_err(|error| {
        format!("{HEARTBEAT_INTERVAL_ENV} must be an integer number of seconds: {error}")
    })?;
    if seconds == 0 {
        return Ok(None);
    }

    Ok(Some(Duration::from_secs(seconds)))
}

fn run_heartbeat(user: &str, credential: &str, connect_string: &str) -> Result<(), String> {
    let connection = Connection::connect(user, credential, connect_string)
        .map_err(|error| format!("connect to Oracle catalog for heartbeat: {error}"))?;
    let value: i64 = connection
        .query_row_as("select 1 from dual", &[])
        .map_err(|error| format!("run Oracle catalog heartbeat query: {error}"))?;
    if value != 1 {
        return Err(format!(
            "Oracle catalog heartbeat returned {value}, expected 1"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_heartbeat_defaults_to_daily_interval() {
        assert_eq!(
            parse_heartbeat_interval(None).unwrap(),
            Some(Duration::from_secs(24 * 60 * 60))
        );
        assert_eq!(
            parse_heartbeat_interval(Some("   ")).unwrap(),
            Some(Duration::from_secs(24 * 60 * 60))
        );
    }

    #[test]
    fn oracle_heartbeat_zero_disables() {
        assert_eq!(parse_heartbeat_interval(Some("0")).unwrap(), None);
    }

    #[test]
    fn oracle_heartbeat_accepts_custom_interval() {
        assert_eq!(
            parse_heartbeat_interval(Some("3600")).unwrap(),
            Some(Duration::from_secs(3600))
        );
    }

    #[test]
    fn oracle_heartbeat_rejects_invalid_interval() {
        let error = parse_heartbeat_interval(Some("daily")).unwrap_err();
        assert!(error.contains(HEARTBEAT_INTERVAL_ENV));
        assert!(error.contains("integer number of seconds"));
    }
}
