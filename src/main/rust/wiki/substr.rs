#[allow(unused)]
use crate::wiki as m;

pub(crate) trait SubStr {
  fn substr_by_symbols<'a>(&'a self, begin: usize, length: usize) -> &'a Self;
  fn remain_by_symbols<'a>(&'a self, begin: usize) -> &'a Self;
}

impl SubStr for str {
  fn substr_by_symbols<'a>(&'a self, begin: usize, length: usize) -> &'a Self {
    let str_begin = self.chars().take(begin).fold(0, |acc, val| acc + val.len_utf8());
    let length = self.chars().skip(begin).take(length).fold(0, |acc, val| acc + val.len_utf8());
    &self[str_begin..str_begin + length]
    // unsafe {
    //   std::str::from_utf8_unchecked(&self.as_bytes()[begin..begin+length])
    // }
  }

  fn remain_by_symbols<'a>(&'a self, begin: usize) -> &'a Self {
    let begin = self.chars().take(begin).fold(0, |acc, val| acc + val.len_utf8());
    &self[begin..]
  }
}
