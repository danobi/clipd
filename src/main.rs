use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod frame;
use frame::{RequestFrame, ResponseFrame};

#[derive(StructOpt, Debug)]
struct Opt {
    /// Port to run server on
    #[structopt(short, long, default_value = "3399")]
    port: u16,
}

/// Handles a single client/connection
///
/// Note that we do not return any errors here. If there are errors, we log
/// it and return. Can't really do anything else.
async fn handle_client(mut client: TcpStream, addr: SocketAddr, clipboard: Arc<Mutex<Vec<u8>>>) {
    let mut buf = Vec::with_capacity(4096);
    match client.read_to_end(&mut buf).await {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to read from {}: {}", addr, e);
            return;
        }
    };

    let request = match RequestFrame::from_bytes(&buf) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse request from {}: {}", addr, e);
            return;
        }
    };

    let response = match request {
        RequestFrame::Push(data) => {
            let mut clipboard = clipboard.lock().unwrap();
            *clipboard = data;
            ResponseFrame::PushOk
        }
        RequestFrame::Pull => {
            let clipboard = clipboard.lock().unwrap();
            ResponseFrame::PullOk(clipboard.clone())
        }
    };

    let response_bytes = response.to_bytes();
    match client.write_all(&response_bytes).await {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to write reponse to to {}: {}", addr, e);
            return;
        }
    };
}

async fn serve(port: u16) -> Result<()> {
    let clipboard: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::with_capacity(4096)));
    let listener = TcpListener::bind(("localhost", port)).await?;
    info!("Listening on: localhost:{}", port);

    loop {
        // Handle clients concurrently
        let (socket, client_addr) = listener.accept().await?;
        let clipboard = clipboard.clone();
        tokio::spawn(async move { handle_client(socket, client_addr, clipboard).await });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    serve(opts.port).await
}
