#[allow(unused)]
use crate::wiki as m;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Remote(#[from] remote::Error),
  #[error(transparent)]
  Database(#[from] database::Error),
}
