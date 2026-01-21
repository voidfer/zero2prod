use zero2prod::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let server = run(listener).await?;
    server.await
}
