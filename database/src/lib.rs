use std::fs::File;

fn hash(source: &str) -> u32 {
  let mut summ = 0;
  for c in source.chars() {
    summ += c as u32;
  }
  summ
}

pub struct MainFile {
  hash: u32,
  word: u64,
  value: u64,
  w_length: u16,
  v_length: u16,
}

pub struct PropertyHeadFile {

}

pub struct Base {
  main_file: Option<File>,
  value_file: Option<File>,
}
