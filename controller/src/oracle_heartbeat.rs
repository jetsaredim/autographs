use std::{sync::Arc, time::Duration};

use tokio::time::{self, MissedTickBehavior};

use crate::oracle_connection::{self, OracleConnectionSettings};

const HEARTBEAT_INTERVAL_ENV: &str = "AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS";
const DEFAULT_HEARTBEAT_INTERVAL_SECONDS: u64 = 24 * 60 * 60;

pub fn spawn(settings: Arc<OracleConnectionSettings>) -> Result<(), String> {
    let Some(interval) = heartbeat_interval_from_env()? else {
        tracing::info!("Oracle catalog heartbeat disabled");
        return Ok(());
    };

    tracing::info!(
        user = settings.user(),
        interval_seconds = interval.as_secs(),
        "starting Oracle catalog heartbeat"
    );

    tokio::spawn(async move {
        let mut ticker = heartbeat_ticker(interval);

        loop {
            ticker.tick().await;

            let result = run_heartbeat(Arc::clone(&settings)).await;

            match result {
                Ok(()) => tracing::info!("Oracle catalog heartbeat succeeded"),
                Err(error) => {
                    tracing::warn!(error_kind = error.kind(), "Oracle catalog heartbeat failed");
                }
            }
        }
    });

    Ok(())
}

fn heartbeat_ticker(interval: Duration) -> time::Interval {
    let mut ticker = time::interval_at(time::Instant::now(), interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker
}

fn heartbeat_interval_from_env() -> Result<Option<Duration>, String> {
    // ast-grep-ignore: no-distributed-env-read
    match std::env::var(HEARTBEAT_INTERVAL_ENV) {
        Ok(value) => parse_heartbeat_interval(Some(value.as_str())),
        Err(std::env::VarError::NotPresent) => parse_heartbeat_interval(None),
        Err(std::env::VarError::NotUnicode(_)) => {
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

async fn run_heartbeat(settings: Arc<OracleConnectionSettings>) -> Result<(), HeartbeatError> {
    let connection = oracle_connection::connect_with_recovery(settings)
        .await
        .map_err(|_error| HeartbeatError::Connect)?;
    tokio::task::spawn_blocking(move || {
        let row = connection
            .query_row("select 1 from dual", &[])
            .map_err(|_error| HeartbeatError::Query)?;
        let value: i64 = row.get(0).map_err(|_error| HeartbeatError::Query)?;
        if value != 1 {
            return Err(HeartbeatError::UnexpectedResult);
        }

        Ok(())
    })
    .await
    .map_err(|_error| HeartbeatError::Task)?
}

enum HeartbeatError {
    Connect,
    Query,
    Task,
    UnexpectedResult,
}

impl HeartbeatError {
    fn kind(&self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::Query => "query",
            Self::Task => "task",
            Self::UnexpectedResult => "unexpected_result",
        }
    }
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

    #[tokio::test(start_paused = true)]
    async fn oracle_heartbeat_ticker_ticks_immediately_then_repeats_on_interval() {
        let interval = Duration::from_secs(60);
        let mut ticker = heartbeat_ticker(interval);

        ticker.tick().await;

        let second_tick = ticker.tick();
        tokio::pin!(second_tick);
        tokio::select! {
            _ = &mut second_tick => panic!("second heartbeat tick fired before interval"),
            _ = tokio::task::yield_now() => {}
        }

        time::advance(interval).await;
        second_tick.await;
    }

    #[test]
    fn oracle_heartbeat_rejects_invalid_interval() {
        let error = parse_heartbeat_interval(Some("daily")).unwrap_err();
        assert!(error.contains(HEARTBEAT_INTERVAL_ENV));
        assert!(error.contains("integer number of seconds"));
    }
}
