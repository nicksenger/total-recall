#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate serde_derive;

pub mod db;
pub mod graphql;
pub mod service;

use juniper::{ExecutionResult, FieldError};
use std::{error, fmt};
use wundergraph::scalar::WundergraphScalarValue;

#[derive(Debug)]
pub enum TRCError {
  Request(reqwest::Error),
  FileSystem(std::io::Error),
  Unauthorized,
  Unknown(String),
}

impl fmt::Display for TRCError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      TRCError::Request(ref err) => write!(f, "Request error: {}", err),
      TRCError::FileSystem(ref err) => write!(f, "File system error: {}", err),
      TRCError::Unauthorized => write!(f, "Unauthorized"),
      TRCError::Unknown(ref err) => write!(f, "Unknown error: {}", err),
    }
  }
}

impl error::Error for TRCError {
  fn cause(&self) -> Option<&(dyn error::Error)> {
    match *self {
      TRCError::Request(ref err) => Some(err),
      TRCError::FileSystem(ref err) => Some(err),
      _ => None,
    }
  }
}

impl From<std::io::Error> for TRCError {
  fn from(err: std::io::Error) -> TRCError {
    TRCError::FileSystem(err)
  }
}

impl From<reqwest::Error> for TRCError {
  fn from(err: reqwest::Error) -> TRCError {
    TRCError::Request(err)
  }
}

impl From<TRCError> for ExecutionResult<WundergraphScalarValue> {
  fn from(err: TRCError) -> ExecutionResult<WundergraphScalarValue> {
    Err(FieldError::new(
      format!("{}", err.to_string()),
      graphql_value!({
          "type": "INTERNAL"
      }),
    ))
  }
}
