use std::num::{ParseFloatError, ParseIntError};

use nom::error::{ErrorKind, ParseError};

#[derive(Debug)]
pub enum PhpError<I> {
  Null,
  BadSeparator(I),
  UnknownPrefix(I),
  BadLength(u32, u32),
  Parse(Box<dyn std::error::Error>),
  Nom(I, ErrorKind),
}
impl<I> PhpError<I> {
  pub fn null() -> Self {
    Self::Null
  }
}
impl<I> From<ParseIntError> for PhpError<I> {
  fn from(src: ParseIntError) -> Self {
    Self::Parse(Box::new(src))
  }
}
impl<I> From<ParseFloatError> for PhpError<I> {
  fn from(src: ParseFloatError) -> Self {
    Self::Parse(Box::new(src))
  }
}
impl<I> ParseError<I> for PhpError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) ->  Self {
    Self::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}
impl<I> From<PhpError<I>> for nom::Err<PhpError<I>> {
  fn from(src: PhpError<I>) -> Self {
    nom::Err::Error(src)
  }
}
