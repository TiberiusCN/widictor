#[allow(unused)]
use crate::wiki as m;

#[derive(Debug)]
enum Element {
  Language(Language),
  WordSection(WordSection),
  Text(Text),
  LanguageSeparator,
}
