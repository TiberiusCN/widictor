#[allow(unused)]
use crate::wiki as m;

#[derive(Debug)]
struct Wiki<'lang> {
  word: String,
  words: HashSet<String>,
  languages: &'lang [String],
}

impl<'lang> Wiki<'lang> {
  named!(section_ending<&str, &str, WikiError<&str>>, tag!("----"));
  named!(line_ending<&str, &str, WikiError<&str>>, tag!("\n"));
  named!(line<&str, &str, WikiError<&str>>, take_until!("\n"));

  pub fn new(page: &str, languages: &'lang [String]) -> Self {
    let mut words = HashSet::new();
    words.insert(page.to_owned());
    Self {
      word: String::new(),
      words,
      languages,
    }
  }

  fn parse(&mut self, bases: &Bases, generation: u32) -> Result<bool, Error> {
    self.word = if let Some(word) = self.words.iter().next() {
      word.clone()
    } else {
      return Ok(false);
    };
    self.words.remove(&self.word);

    if self.word.contains(' ') {
      for word in self.word.split(' ') {
        self.words.insert(word.to_owned());
      }
    }

    if self.word.contains(':') {
      println!("\x1b[33m{}\x1b[0m", &self.word);
      return Ok(true);
    }

    let gen = bases.total(|base| {
      base.search_word(&self.word)
    }).unwrap().unwrap_or_default();
    if gen < generation {
      println!("\x1b[32m{}\x1b[0m", &self.word);

      let data = remote::get(&self.word);
      if let Err(remote::Error::Reqwest(..)) = &data {
      } else {
        bases.total(|base| {
          if gen == 0 {
            base.insert_word(&self.word, generation)
          } else {
            base.update_word(&self.word, generation)
          }
        })?;
      }

      let data = data?;

      let mut input = data.as_str();

      let mut elements: Vec<Element> = Vec::new();

      let mut subs = HashSet::new();

      let mut wrap = |source: &str| {
        if source.is_empty() { return; }

        let element =
          Language::parse(source).map(|s| Element::Language(s.1))
          .or_else(|_| WordSection::parse(source).map(|s| Element::WordSection(s.1)))
          .or_else(|_| Self::section_ending(source).map(|_| Element::LanguageSeparator))
          .or_else(|_| Text::parse_any(source, &mut subs).map(|s| Element::Text(s.1)))
          .unwrap();

        elements.push(element);
      };

      while let Ok(line) = Self::line(input) {
        input = Self::line_ending(line.0).unwrap().0;
        wrap(line.1);
      }
      wrap(input);

      let mut combinator = Combinator::default();

      for section in elements {
        match section {
          Element::Language(language) => combinator.push_language(language),
          Element::WordSection(section) => combinator.push_section(section),
          Element::Text(text) => combinator.push_text(text),
          Element::LanguageSeparator => {},
        }
      }

      let combinator = combinator.finish();

      let (words, subwords) = { combinator.build(&self) };
      for word in subwords {
        self.words.insert(word);
      }
      for sub in subs {
        self.words.insert(sub);
      }

      for word in words {
        if let Ok(mut base) = bases.load_language(&word.0).map_err(|e| log::error!("{}", e)) {
          word.1.into_iter().map(|lemma| -> Result<_, _> {
            let mut errors = Vec::new();
            let mut word = base.insert_word(&self.word, lemma.value.as_ref().unwrap()).map_err(|e| vec![e])?;
            for tag in lemma.tags {
              if let Err(e) = word.insert_tag(&tag) { errors.push(e); }
            }
            for property in lemma.properties {
              if let Err(e) = word.insert_property(&property.0, &property.1) { errors.push(e); }
            }
            for term in lemma.produced {
              if let Err(e) = word.insert_produced(&term) { errors.push(e); }
            }
            for term in lemma.derived {
              if let Err(e) = word.insert_derived(&term) { errors.push(e); }
            }
            if let Some(mutation) = lemma.mutation {
              for form in mutation {
                if let Err(e) = word.insert_form(&form.0, &form.1) { errors.push(e); }
              }
            }
            Ok(())
          }).for_each(|e: Result<_, Vec<database::Error>>| {
            if let Err(e) = e {
              for e in e {
                log::error!("{}", e);
              }
            }
          });
        }
      }
    } else {
      println!("\x1b[31m{}\x1b[0m", &self.word);
    }
    Ok(!self.words.is_empty())
  }
}
