use autographs_controller::{config::ControllerConfig, routes::runtime_router};

#[tokio::main]
async fn main() {
    let config = ControllerConfig::from_env().expect("load controller configuration");
    let bind_addr = config.bind_addr;
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("bind controller listener");

    axum::serve(
        listener,
        runtime_router(config).expect("configure controller persistence"),
    )
    .await
    .expect("serve controller routes");
}
