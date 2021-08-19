#[allow(unused)]
use crate::scribunto as m;

use nom::error::{ErrorKind, ParseError};
use std::{fmt::{Debug, Display}, num::{ParseFloatError, ParseIntError}};

#[derive(thiserror::Error, Debug)]
pub enum PhpError<I: Display + Debug> {
  #[error("bad length: found {0}, expected {1}")]
  BadLength(u32, u32),
  #[error("bad type")]
  BadType,
  #[error("bad prefix: found {1}, expected {0}")]
  UnexpectedPrefix(&'static str, String),
  #[error("parse error: {0}")]
  Parse(Box<dyn std::error::Error>),
  #[error("lua error: {0}")]
  Lua(String),
  #[error("unknown op: {0}")]
  UnknownOp(String),
  #[error("nom error: {1:?} ({0})")]
  Nom(I, ErrorKind),
  #[error("no such function: {0}")]
  NoSuchFunction(String),
}
impl<I: Display + Debug> PhpError<I> {
  pub fn into_nom(self) -> nom::Err<Self> {
    self.into()
  }
}
impl<I: Display + Debug> From<ParseIntError> for PhpError<I> {
  fn from(src: ParseIntError) -> Self {
    Self::Parse(Box::new(src))
  }
}
impl<I: Display + Debug> From<ParseFloatError> for PhpError<I> {
  fn from(src: ParseFloatError) -> Self {
    Self::Parse(Box::new(src))
  }
}
impl<I: Display + Debug> ParseError<I> for PhpError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) ->  Self {
    Self::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}
impl<I: Display + Debug> From<PhpError<I>> for nom::Err<PhpError<I>> {
  fn from(src: PhpError<I>) -> Self {
    nom::Err::Error(src)
  }
}
