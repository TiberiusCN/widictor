#[derive(Debug, Default)]
struct Combinator {
  article_texts: Vec<Text>,
  languages: HashMap<String, Language>,
  sections: Vec<Vec<WordSection>>,
  texts: Vec<Text>,

  last_language: Option<String>,
}

impl Combinator {
  fn compact_texts(&mut self) -> &mut Self {
    if let Some(section) = self.sections.last_mut().and_then(|s| s.last_mut()) {
      section.content.append(&mut self.texts);
    } else if let Some(language) = self.last_language.as_ref() {
      self.languages.get_mut(language).unwrap().content.append(&mut self.texts);
    } else {
      self.article_texts.append(&mut self.texts);
    }

    self
  }

  fn compact_sections(&mut self, level: usize) -> &mut Self {
    self.compact_texts();

    while self.sections.len() > level + 1 {
      let mut lasts = self.sections.pop().unwrap();
      self.sections.last_mut().unwrap().last_mut().unwrap().sections.append(&mut lasts);
    }

    self
  }

  fn compact_language(&mut self) -> &mut Self {
    self.compact_sections(0);

    if let Some(target_language) = self.last_language.as_ref() {
      let target_language = self.languages.get_mut(target_language).unwrap();
      if let Some(mut sections) = self.sections.pop() {
        target_language.sections.append(&mut sections);
      }
    }

    self
  }

  fn finish(mut self) -> Self {
    self.compact_language().last_language = None;
    self
  }

  fn push_language(&mut self, language: Language) {
    let name = language.name.clone();
    self.compact_language().languages.insert(language.name.clone(), language);
    self.last_language = Some(name);
  }

  fn push_section(&mut self, section: WordSection) {
    let mut level = section.level;
    self.compact_sections(level);
    if self.sections.len() <= level {
      self.sections.push(Vec::new());
    }
    if self.sections.len() <= level {
      level = self.sections.len() - 1;
    }
    self.sections[level].push(section);
  }

  fn push_text(&mut self, text: Text) {
    self.texts.push(text);
  }

  fn build<'a>(&self, wiki: &'a Wiki) -> (HashMap<String, Vec<Lemma>>, HashSet<String>) {
    if self.last_language != None { panic!("unfinished"); }

    let mut out_words = HashMap::new();
    let mut out_subwords = HashSet::new();

    for language in wiki.languages {
      let mut words = Vec::new();

      if let Some(lang_value) = self.languages.get(language) {
        for sections in lang_value.build().into_iter() {
          let mut word = Lemma::default();

          for section in &sections.sections {
            if let Some(section) = section.as_ref() {
              let section = &section.0;
              let mut lemma = section.text(wiki);
              let value = lemma.value.take();
              if let Some(value) = value {
                if let Some(species) = section.name.general_species() {
                  match species {
                    SectionSpecies::Unknown => {},
                    SectionSpecies::Word => {
                      let mut out = String::new();
                      for mut line in value.lines() {
                        let mut deep = 0;
                        while let Some(subline) = line.strip_prefix("*") {
                          deep += 1;
                          line = subline;
                        }
                        line = line.trim();
                        if deep == 2 && line.ends_with(':') {
                          continue; // quotation source
                        }
                        let mut new_line = String::new();
                        let mut quote = 0u32;
                        let mut is_begin = true;
                        for c in line.chars() {
                          if c == '\'' {
                            quote += 1;
                          } else {
                            match quote {
                              0 => {},
                              1 => new_line.push('\''),
                              2 => new_line.push('"'),
                              3 => new_line.push('_'),
                              4 => {}, // '' [ ] ''
                              5 => if is_begin {
                                new_line += "_\"";
                              } else {
                                new_line += "\"_";
                              },
                              6 => if is_begin {
                                new_line += "_\"'";
                              } else {
                                new_line += "'\"_";
                              },
                              c => panic!("overquoted: {} in {} … {}", c, out, line),
                            }
                            quote = 0;
                            new_line.push(c);
                          }
                          is_begin = c.is_whitespace();
                        }
                        match quote {
                          0 => {},
                          1 => new_line.push('\''),
                          2 => new_line.push('"'),
                          3 => new_line.push('_'),
                          4 => {},
                          5 => if is_begin {
                            new_line += "_\"";
                          } else {
                            new_line += "\"_";
                          },
                          6 => if is_begin {
                            new_line += "_\"'";
                          } else {
                            new_line += "'\"_";
                          },
                          c => panic!("overquoted: {} in {} … {}", c, out, line),
                        }
                        if !new_line.is_empty() {
                          if deep > 1 {
                            out.push('{');
                            out.push('[');
                          }
                          out += &new_line;
                          if deep > 1 {
                            out.push(']');
                            out.push('}');
                          }
                          out.push('\n');
                        }
                      }
                      if let Some(l) = out.pop() {
                        if l != '\n' {
                          out.push('\n');
                        }
                      }
                      lemma.value = Some(out);
                    },
                    SectionSpecies::Etymology => {
                      lemma.properties.insert("etymology".to_owned(), value);
                    },
                    SectionSpecies::Mutation => {
                      lemma.properties.insert("mutation notes".to_owned(), value);
                    },
                    SectionSpecies::Pronunciation => {
                      // lemma.properties.insert("pronunciation".to_owned(), value); },
                    },
                    SectionSpecies::Provided => {
                    },
                    SectionSpecies::UsageNotes => {
                      lemma.properties.insert("usage notes".to_owned(), value);
                    },
                  }
                }
              }
              if let Some(tag) = section.name.tag() {
                lemma.tags.insert(tag.to_owned());
              }
              word += lemma;
            }
          }
          for subword in word.derived.iter().chain(word.produced.iter()) {
            println!("\x1b[35m{}\x1b[0m", subword);
            out_subwords.insert(subword.to_owned());
          }
          if word.value.is_some() {
            words.push(word);
          }
        }
      }
      out_words.insert(language.clone(), words);
    }
    (out_words, out_subwords)
  }
}
