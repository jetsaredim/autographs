use autographs_controller::{config::ControllerConfig, routes::runtime_router};

#[tokio::main]
async fn main() {
    init_logging();

    let config = ControllerConfig::from_env().expect("load controller configuration");
    let bind_addr = config.bind_addr;
    tracing::info!(%bind_addr, "starting autographs controller");

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("bind controller listener");

    tracing::info!(%bind_addr, "controller listener bound");

    axum::serve(
        listener,
        runtime_router(config).expect("configure controller persistence"),
    )
    .await
    .expect("serve controller routes");
}

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("autographs_controller=info, tower_http=info"));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(true))
        .init();
}
