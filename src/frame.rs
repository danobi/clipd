use std::convert::TryInto;

use anyhow::{bail, Result};

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
const CLIPD_MAGIC: u8 = 0b10010101;

const REQUEST_PUSH: u8 = 55;
const REQUEST_PULL: u8 = 56;

pub enum RequestFrame {
    /// A request to push bytes to the server
    Push(Vec<u8>),
    /// A request to pull the currently stored bytes on the server
    Pull,
}

pub enum RequestFramePeekLen {
    /// Payload length is enclosed
    ///
    /// 0 if no payload
    Len(u64),
    /// Not enough bytes of the header were passed in
    NotEnoughBytes,
}

fn validate_magic(bytes: &[u8]) -> Result<()> {
    let magic = bytes[0]; // u8 does not have endianness
    if magic != CLIPD_MAGIC {
        bail!("Provided magic ({}) != magic ({})", magic, CLIPD_MAGIC);
    }

    Ok(())
}

impl RequestFrame {
    /// Peek how many bytes long the payload is
    pub fn peek_len(bytes: &[u8]) -> Result<RequestFramePeekLen> {
        if bytes.len() < 10 {
            return Ok(RequestFramePeekLen::NotEnoughBytes);
        }

        validate_magic(bytes)?;

        let ty = bytes[1]; // u8 does not have endianness
        if ty == REQUEST_PUSH {
            let len = u64::from_be_bytes(bytes[2..10].try_into().unwrap());
            return Ok(RequestFramePeekLen::Len(len));
        }

        return Ok(RequestFramePeekLen::Len(0));
    }

    /// Construct a `RequestFrame` from bytes on the wire
    pub fn from_bytes(bytes: &[u8]) -> Result<RequestFrame> {
        if bytes.len() < 2 {
            bail!("Request must be at least 2 bytes for magic+type");
        }

        validate_magic(bytes)?;

        let ty = bytes[1]; // u8 does not have endianness
        let req = match ty {
            REQUEST_PUSH => {
                let len = u64::from_be_bytes(bytes[2..10].try_into().unwrap());
                if (bytes.len() - 10) as u64 != len {
                    bail!("REQUEST_PUSH len != payload len");
                }

                RequestFrame::Push(bytes[10..].to_vec())
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
