#[derive(Debug)]
struct Language {
  name: String,
  sections: Vec<WordSection>,
  content: Vec<Text>,
}

impl Language {
  named!(language<&str, &str, WikiError<&str>>, delimited!(tag!("=="), take_while1!(|c: char| c.is_alphabetic() || c.is_whitespace()), tag!("==")));

  fn parse(input: &str) -> IResult<&str, Self, WikiError<&str>> {
    let value = Self::language(input)?;

    Ok((value.0, Self {
      name: value.1.to_string(),
      content: Vec::new(),
      sections: Vec::new(),
    }))
  }

  fn build(&self) -> Vec<Word> {
    let mut builder = vec![Word::default()];
    if !self.sections.is_empty() {
      for section in &self.sections {
        builder = section.build(builder, 0);
      }
    }
    builder
  }
}
