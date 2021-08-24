#[allow(unused)]
use crate::wiki as m;

use nom::error::{ErrorKind, ParseError};

#[derive(Debug)]
pub enum WikiError<I> {
  TemplateHasNoHeader,
  BadTemplate,
  OpenNotMatchesClose,
  UnexpectedTail(I),
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for WikiError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    Self::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl<I> From<WikiError<I>> for nom::Err<WikiError<I>> {
  fn from(src: WikiError<I>) -> Self {
    nom::Err::Error(src)
  }
}

impl<I> WikiError<I> {
  pub fn filtered(&self) -> bool {
    matches!(self, Self::Nom(_, _))
  }
}
