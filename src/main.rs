use std::net;

use zero2prod::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = net::TcpListener::bind("localhost:8000")?;
    run(listener)?.await
}
