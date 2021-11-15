use anyhow::{bail, Result};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

// clipd protocol (request and response):
//
// 0       8      16
// +-------+-------------------------+
// | magic | type | optional payload |
// +-------+-------------------------+
//
// The payload, if present:
//
// 0     64
// +-----+---------+
// | len | payload |
// +-----+---------+

/// Magic for all frames
///
/// Note that u8's don't have endianness so we don't need to swap bytes
const CLIPD_MAGIC: u8 = 0b10010101;

const REQUEST_PUSH: u8 = 55;
const REQUEST_PULL: u8 = 56;

pub enum RequestFrame {
    /// A request to push bytes to the server
    Push(Vec<u8>),
    /// A request to pull the currently stored bytes on the server
    Pull,
}

impl RequestFrame {
    /// Construct a `RequestFrame` from bytes on the wire
    pub async fn from_socket(socket: &mut TcpStream) -> Result<RequestFrame> {
        // All requests will have the same 2 byte header, so grab that first
        // and switch based on the request type
        let mut header = [0; 2];
        socket.read_exact(&mut header).await?;

        // Toss out invalid requests (common for port scanners to trip over)
        if header[0] != CLIPD_MAGIC {
            bail!("Provided magic ({}) != magic ({})", header[0], CLIPD_MAGIC);
        }

        let ty = header[1];
        let req = match ty {
            REQUEST_PUSH => {
                // Grab the length of the payload
                let mut len = [0; 8];
                socket.read_exact(&mut len).await?;
                let len = u64::from_be_bytes(len);

                // Hard cap the payload to be 10M
                if len > (10 << 20) {
                    bail!("Payload too large ({}MB)", len >> 20);
                }

                // Read the payload
                let mut payload = vec![0; len as usize];
                socket.read_exact(&mut payload).await?;

                RequestFrame::Push(payload)
            }
            REQUEST_PULL => RequestFrame::Pull,
            _ => bail!("Unrecognized request type: {}", ty),
        };

        Ok(req)
    }

    /// Prepare a `RequestFrame` for transport on the wire
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Push(bytes) => {
                let mut r = Vec::with_capacity(2 + bytes.len());
                r.push(CLIPD_MAGIC.to_be());
                r.push(REQUEST_PUSH.to_be());
                r.extend((bytes.len() as u64).to_be_bytes());
                r.extend(bytes);

                r
            }
            Self::Pull => {
                vec![CLIPD_MAGIC.to_be(), REQUEST_PULL.to_be()]
            }
        }
    }
}

const RESPONSE_PUSH_OK: u8 = 100;
const RESPONSE_PUSH_ERR: u8 = 101;
const RESPONSE_PULL_OK: u8 = 102;
const RESPONSE_PULL_ERR: u8 = 103;

pub enum ResponseFrame {
    /// Push to server was OK
    PushOk,
    /// Push to server errored. UTF-8 encoded error message enclosed
    PushErr(Vec<u8>),
    /// Pull was OK. Payload is enclosed
    PullOk(Vec<u8>),
    /// Pull from server errored. UTF-8 encoded error message enclosed
    PullErr(Vec<u8>),
}

impl ResponseFrame {
    pub fn from_bytes(bytes: &[u8]) -> Result<ResponseFrame> {
        unimplemented!();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unimplemented!();
    }
}
