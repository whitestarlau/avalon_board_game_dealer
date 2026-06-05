fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        match backend::start_server(3004).await {
            Ok(url) => println!("Server running at {} (http://127.0.0.1:3004)", url),
            Err(e) => eprintln!("Server failed: {}", e),
        }
        tokio::signal::ctrl_c().await.ok();
        backend::stop_server().await;
    });
}
