pub mod core;
pub mod example;
pub mod inmemory_db;
pub mod server;

#[tokio::main]
async fn main() {
    example::run().await
}
