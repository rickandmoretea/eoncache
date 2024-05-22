use crate::Frame;
use bytes::Bytes;
use std::{fmt, vec};

/// Utility for parsing a command
#[derive(Debug)]
pub struct Parse {
    parts: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub enum ParseError {
    EndOfStream,
    Other(String),
}

impl Parse {
    pub(crate) fn new(frame: Frame) -> Result<Parse, ParseError> {
        match frame {
            Frame::Array(array) => Ok(Parse {
                parts: array.into_iter(),
            }),
            _ => Err(ParseError::Other("Expected array frame".into())),
        }
    }

    pub fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or(ParseError::EndOfStream)
    }

    pub fn next_string(&mut self) -> Result<String, ParseError> {
        let bytes = match self.next()? {
            Frame::Simple(s) => s.into_bytes(),
            Frame::Bulk(b) => b.to_vec(),
            _ => return Err(ParseError::Other("Expected simple or bulk frame".to_string())),
        };

        // Attempt to convert bytes to a UTF-8 string
        String::from_utf8(bytes).map_err(|_| ParseError::Other("Invalid UTF-8".to_string()))
    }

    pub fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Frame::Bulk(b) => Ok(b),
            _ => Err(ParseError::Other("Expected simple or bulk frame".to_string())),
        }
    }

    pub fn next_int(&mut self) -> Result<u64, ParseError> {
        let s = self.next_string()?;
        s.parse::<u64>().map_err(|_| ParseError::Other("Invalid integer".to_string()))
    }

    pub fn finish(&mut self) -> Result<(), ParseError> {
        if self.parts.next().is_some() {
            Err(ParseError::Other("Extra data in frame".to_string()))
        } else {
            Ok(())
        }
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> Self {
        ParseError::Other(src)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EndOfStream => write!(f, "Unexpected end of stream"),
            ParseError::Other(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for ParseError {}
