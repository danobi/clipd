use std::io::{self, Read};

use anyhow::{bail, Result};
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use clipd::frame::{RequestFrame, ResponseFrame};

#[derive(StructOpt, Debug)]
struct Opt {
    /// clipd server to connect to
    #[structopt(short, long, default_value = "localhost")]
    server: String,
    /// Port to run server on
    #[structopt(short, long, default_value = "3399")]
    port: u16,
    /// Pull remote clipboard
    #[structopt(long)]
    pull: bool,
}

async fn pull(socket: &mut TcpStream) -> Result<()> {
    // Send request
    let req = RequestFrame::Pull;
    socket.write_all(&req.to_bytes()).await?;

    // Receive response
    let resp = ResponseFrame::from_socket(socket).await?;
    match resp {
        ResponseFrame::PushOk => bail!("Unexpected response: PushOk"),
        ResponseFrame::PushErr(_) => bail!("Unexpected response: PushErr"),
        ResponseFrame::PullOk(bytes) => {
            let payload = String::from_utf8(bytes)?;
            println!("{}", payload);
            Ok(())
        }
        ResponseFrame::PullErr(bytes) => {
            let err = String::from_utf8(bytes)?;
            bail!("Error pulling from server: {}", err);
        }
    }
}

async fn push(socket: &mut TcpStream) -> Result<()> {
    // Grab stdin input
    let mut input = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut input)?;

    // Send request
    let req = RequestFrame::Push(input.into_bytes());
    socket.write_all(&req.to_bytes()).await?;

    // Receive response
    let resp = ResponseFrame::from_socket(socket).await?;
    match resp {
        ResponseFrame::PushOk => Ok(()),
        ResponseFrame::PushErr(bytes) => {
            let err = String::from_utf8(bytes)?;
            bail!("Error pushing to server: {}", err);
        }
        ResponseFrame::PullOk(_) => bail!("Unexpected response: PullOk"),
        ResponseFrame::PullErr(_) => bail!("Unexpected response: PullErr"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();

    let mut socket = TcpStream::connect(format!("{}:{}", opts.server, opts.port)).await?;
    if opts.pull {
        pull(&mut socket).await
    } else {
        push(&mut socket).await
    }
}
