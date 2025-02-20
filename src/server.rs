// src/server.rs

    use axum::Router;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use crate::config::{get_host, get_port};

    fn parse_socket_addr(host: &str, port: u16) -> SocketAddr {
        SocketAddr::new(host.parse().expect("Invalid host address"), port)
    }

    pub async fn run_server(app: Router) {
        // Set up the server address
        let addr = parse_socket_addr(&get_host(), get_port());

        println!("Server running on http://{}", addr);

        // Create a TCP listener
        let listener = TcpListener::bind(addr).await.expect("Failed to bind address");
        println!("Listening on {}", addr);

        // Start serving
        axum::serve(listener, app).await.expect("Server error");
    }