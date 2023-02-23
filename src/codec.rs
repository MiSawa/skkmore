use std::{fmt::Display, io::Write};

use bytes::{Buf, BufMut, BytesMut};
use clap::{crate_name, crate_version};
use joinery::JoinableIterator;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum Request {
    /// Command `0`. The TCP connection should be closed.
    CloseConnection,
    /// Command `1`. Request for conversion.
    Convert(String),
    /// Command `2`. Request for version string.
    GetVersion,
    /// Command `3`. Request for host info.
    GetHostInfo,
    /// Command `4`. Request for completion.
    Complete(String),
}

#[derive(Debug)]
pub enum Response {
    /// Response for command `1` and `4`.
    Candidates(Vec<String>),
    /// Response for command `2`.
    Version,
    /// Response for command `3`.
    HostInfo,
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Unknown command {0}u8")]
    UnknownCommand(u8),
    #[error(transparent)]
    EncodingError(#[from] std::str::Utf8Error),
}

pub struct RequestCodec;
impl Decoder for RequestCodec {
    type Item = Request;

    type Error = ProtocolError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        fn try_read_request(src: &mut BytesMut) -> Option<Result<String, ProtocolError>> {
            if let Some(i) = src.iter().position(|c| c == &b' ') {
                src.advance(1); // skip command
                let body = src.split_to(i - 1);
                src.advance(1); // skip space
                Some(
                    std::str::from_utf8(&body[..])
                        .map(str::to_string)
                        .map_err(ProtocolError::EncodingError),
                )
            } else {
                None
            }
        }

        match src[0] {
            b'0' => {
                src.advance(1);
                Ok(Some(Request::CloseConnection))
            }
            b'1' => {
                if let Some(ret) = try_read_request(src) {
                    Ok(Some(Request::Convert(ret?)))
                } else {
                    Ok(None)
                }
            }
            b'2' => {
                src.advance(1);
                Ok(Some(Request::GetVersion))
            }
            b'3' => {
                src.advance(1);
                Ok(Some(Request::GetHostInfo))
            }
            b'4' => {
                if let Some(ret) = try_read_request(src) {
                    Ok(Some(Request::Complete(ret?)))
                } else {
                    Ok(None)
                }
            }
            _ => Err(ProtocolError::UnknownCommand(src[0])),
        }
    }
}

struct EscapingCandidate<'a>(&'a str);
impl Display for EscapingCandidate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.find('/').is_none() {
            return f.write_str(self.0);
        }
        f.write_str(r#"(concat ""#)?;
        self.0.split('/').join_with(r"\057").fmt(f)?;
        f.write_str(r#"")"#)
    }
}

pub struct ResponseCodec;
impl Encoder<Response> for ResponseCodec {
    type Error = ProtocolError;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut writer = dst.writer();
        match item {
            Response::Candidates(candidates) => {
                if candidates.is_empty() {
                    writeln!(&mut writer, "4")?;
                } else {
                    write!(&mut writer, "1/")?;
                    writeln!(
                        &mut writer,
                        "{}/",
                        candidates
                            .iter()
                            .map(|s| EscapingCandidate(s))
                            .join_with('/')
                    )?;
                }
            }
            Response::Version => {
                write!(&mut writer, "{}/{} ", crate_name!(), crate_version!())?;
            }
            Response::HostInfo => {
                write!(&mut writer, "dummy ")?;
            }
        }
        Ok(())
    }
}
