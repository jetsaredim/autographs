use autographs_controller::{config::ControllerConfig, routes::router};

#[tokio::main]
async fn main() {
    let config = ControllerConfig::from_env().expect("load controller configuration");
    let bind_addr = config.bind_addr;
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("bind controller listener");

    axum::serve(listener, router(config))
        .await
        .expect("serve controller routes");
}
