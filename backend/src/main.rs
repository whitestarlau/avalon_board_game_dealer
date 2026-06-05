fn main() {
    let url = backend::start_server_blocking(3004);
    println!("Server running at {} (http://127.0.0.1:3004)", url);
    // Block main thread forever
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
