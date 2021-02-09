#[derive(Debug)]
enum Element {
  Language(Language),
  WordSection(WordSection),
  Text(Text),
  LanguageSeparator,
}
