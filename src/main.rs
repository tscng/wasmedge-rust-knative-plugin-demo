use axum::{routing::get,Router};
use tokio::net::TcpListener;

use wasmedge_process_interface::Command;

#[tokio::main(flavor = "current_thread")]
async fn main() {

    // build service routes
    let app = Router::new()
        .route("/", get(probe))
        .route("/service", get(service));

    // run service on port 8080
    let addr = "0.0.0.0:8080";
    let tcp_listener = TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", addr);
    axum::Server::from_tcp(tcp_listener.into_std().unwrap())
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// return OK with body to pass readiness probe
async fn probe() -> &'static str {
    "Rust service is alive!"
}

// return ls command output
async fn service() -> String {
    let cmd = Command::new("ls").timeout(1000).output();
    format!("$ ls\n{}",str::from_utf8(&cmd.stdout).expect("GET STDOUT ERR"))
}