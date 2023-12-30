use std::net::TcpListener;

use actix_web::{HttpRequest, Responder};

async fn _greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //NOTE: The Server must be awaited and polled to start running. It resolves when it is shuts
    //down
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    zero2prod::run(listener)?.await
}
