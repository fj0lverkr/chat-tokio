mod lib;

use crate::lib::chatserver::ChatServer;

#[tokio::main]
async fn main() {
    let server = ChatServer::new("localhost".to_string(), 8080);
    server.serve().await;
}