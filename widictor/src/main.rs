use std::collections::{HashMap, HashSet};
use nom::*;
use nom::error::*;
use template::{Word as Lemma, Params, SectionSpecies};
use std::io::Read;
use database::*;

lazy_static::lazy_static! {
  static ref TEMPLATES: HashMap<String, std::path::PathBuf> = {
    let config_path = directories::ProjectDirs::from("com", "apqm", "widictor").unwrap().config_dir().join("templates.conf");
    let x_dir = directories::BaseDirs::new().unwrap().executable_dir().unwrap().to_owned();
    let f = std::fs::read_to_string(config_path).unwrap();
    let mut hash = HashMap::new();
    for p in f.lines() {
      let mut  p = p.split('~');
      if let (Some(template), Some(executable)) = (p.next(), p.next()) {
        hash.insert(template.to_owned(), x_dir.join(executable));
      }
    }
    hash
  };
}

#[derive(Debug)]
pub enum WikiError<I> {
  BadTemplate,
  OpenNotMatchesClose,
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for WikiError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) ->  Self {
    Self::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

#[derive(Debug)]
enum Element {
  Language(Language),
  WordSection(WordSection),
  Text(Text),
  LanguageSeparator,
}

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
          .or_else(|_| Text::parse(source, &mut subs).map(|s| Element::Text(s.1)))
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

#[derive(Debug, PartialEq, std::cmp::Eq, std::hash::Hash, Clone, Copy)]
enum Section {
  Unknown,

  Declension,
  DerivedTerms,
  RelatedTerms,
  Descendants,
  SeeAlso,
  Etymology,
  Pronunciation,
  References,
  FurtherReading,
  AlternativeForms,
  Conjugation,
  UsageNotes,
  Translations,
  Anagrams,
  Synonyms,
  Antonyms,
  Determiner,
  Contraction,
  Inflection,
  Compounds,
  
  Conjunction,
  Noun,
  Verb,
  Adjective,
  Participle,
  Preposition,
  Pronoun,
  Interjection,
  Adverb,
  Numeral,
  Particle,
}

impl Section {
  fn species(&self) -> Option<usize> {
    Some(match self {
      Self::Compounds | Self::Conjunction | Self::Noun | Self::Verb | Self::Adjective | Self::Participle | Self::Preposition | Self::Pronoun | Self::Interjection | Self::Adverb | Self::Numeral | Self::Particle => 0,
      Self::Declension | Self::Conjugation | Self::Inflection => 1,
      Self::DerivedTerms => 2,
      Self::RelatedTerms => 3,
      Self::Descendants => 4,
      Self::Etymology => 5,
      Self::Pronunciation => 6,
      Self::UsageNotes => 7,
      Self::Synonyms => 8,
      Self::Antonyms => 9,

      Self::Unknown | Self::SeeAlso | Self::Anagrams | Self::Translations | Self::References | Self::FurtherReading | Self::AlternativeForms | Self::Determiner | Self::Contraction => return None,
    })
  }
  
  fn general_species(&self) -> Option<SectionSpecies> {
    Some(match self {
      Self::Conjunction | Self::Noun | Self::Verb | Self::Adjective | Self::Participle | Self::Preposition | Self::Pronoun | Self::Interjection | Self::Adverb | Self::Numeral | Self::Particle => SectionSpecies::Word,
      Self::Declension | Self::Conjugation | Self::Inflection => SectionSpecies::Mutation,
      Self::Compounds | Self::DerivedTerms | Self::RelatedTerms | Self::Descendants | Self::Synonyms | Self::Antonyms => SectionSpecies::Provided,
      Self::Etymology => SectionSpecies::Etymology,
      Self::Pronunciation => SectionSpecies::Pronunciation,
      Self::UsageNotes => SectionSpecies::UsageNotes,

      Self::Unknown | Self::SeeAlso | Self::Anagrams | Self::Translations | Self::References | Self::FurtherReading | Self::AlternativeForms | Self::Determiner | Self::Contraction => return None,
    })
  }

  fn tag(&self) -> Option<&'static str> {
    match self {
      Self::Unknown | Self::Compounds | Self::Declension | Self::DerivedTerms | Self::RelatedTerms | Self::Descendants | Self::SeeAlso | Self::Etymology | Self::Pronunciation | Self::References | Self::FurtherReading | Self::AlternativeForms | Self::Conjugation | Self::UsageNotes | Self::Translations | Self::Anagrams | Self::Synonyms | Self::Antonyms | Self::Determiner | Self::Contraction | Self::Inflection => None,

      Self::Conjunction => Some("conjunction"),
      Self::Noun => Some("noun"),
      Self::Verb => Some("verb"),
      Self::Adjective => Some("adjective"),
      Self::Participle => Some("participle"),
      Self::Preposition => Some("preposition"),
      Self::Pronoun => Some("pronoun"),
      Self::Interjection => Some("interjection"),
      Self::Adverb => Some("adverb"),
      Self::Numeral => Some("numeral"),
      Self::Particle => Some("particle"),
    }
  }
}

#[derive(Debug, Clone)]
struct WordSection {
  name: Section,
  level: usize,
  sections: Vec<WordSection>,
  content: Vec<Text>,
}

impl WordSection {
  named!(word_section1<&str, &str, WikiError<&str>>, delimited!(tag!("==="), take_while1!(|c: char| c.is_alphanumeric() || c.is_whitespace()), tag!("===")));
  named!(word_section2<&str, &str, WikiError<&str>>, delimited!(tag!("===="), take_while1!(|c: char| c.is_alphanumeric() || c.is_whitespace()), tag!("====")));
  named!(word_section3<&str, &str, WikiError<&str>>, delimited!(tag!("====="), take_while1!(|c: char| c.is_alphanumeric() || c.is_whitespace()), tag!("=====")));
  named!(word_section<&str, (&str, usize), WikiError<&str>>,
         alt!(
           map!(Self::word_section1, |s| { (s, 1) }) |
           map!(Self::word_section2, |s| { (s, 2) }) |
           map!(Self::word_section3, |s| { (s, 3) })
         ));

  fn parse(input: &str) -> IResult<&str, Self, WikiError<&str>> {
    let value = Self::word_section(input)?;
    let tail = value.0;
    let (value, level) = value.1;
    let section = {
      if value.starts_with("Declension") { Section::Declension }
      else if value.starts_with("Derived terms") { Section::DerivedTerms }
      else if value.starts_with("Related terms") { Section::RelatedTerms }
      else if value.starts_with("Descendants") { Section::Descendants }
      else if value.starts_with("See also") { Section::SeeAlso }
      else if value.starts_with("Etymology") { Section::Etymology }
      else if value.starts_with("Pronunciation") { Section::Pronunciation }
      else if value.starts_with("References") { Section::References }
      else if value.starts_with("Further reading") { Section::FurtherReading }
      else if value.starts_with("Alternative forms") { Section::AlternativeForms }
      else if value.starts_with("Conjugation") { Section::Conjugation }
      else if value.starts_with("Usage notes") { Section::UsageNotes }
      else if value.starts_with("Translations") { Section::Translations }
      else if value.starts_with("Anagrams") { Section::Anagrams }
      else if value.starts_with("Conjunction") { Section::Conjunction }
      else if value.starts_with("Synonyms") { Section::Synonyms }
      else if value.starts_with("Antonyms") { Section::Antonyms }
      else if value.starts_with("Determiner") { Section::Determiner }
      else if value.starts_with("Contraction") { Section::Contraction }
      else if value.starts_with("Inflection") { Section::Inflection }
      else if value.starts_with("Compounds") { Section::Compounds }

      else if value.starts_with("Noun") { Section::Noun }
      else if value.starts_with("Proper noun") { Section::Noun }
      else if value.starts_with("Verb") { Section::Verb }
      else if value.starts_with("Adjective") { Section::Adjective }
      else if value.starts_with("Preposition") { Section::Preposition }
      else if value.starts_with("Pronoun") { Section::Pronoun }
      else if value.starts_with("Participle") { Section::Participle }
      else if value.starts_with("Interjection") { Section::Interjection }
      else if value.starts_with("Adverb") { Section::Adverb }
      else if value.starts_with("Numeral") { Section::Numeral }
      else if value.starts_with("Particle") { Section::Particle }

      else { log::warn!("unknown section: {}", value); Section::Unknown }
    };
    Ok((tail, Self {
      name: section,
      level: level - 1,
      sections: Vec::new(),
      content: Vec::new(),
    }))
  }

  fn build(&self, mut builder: Vec<Word>, level: usize) -> Vec<Word> {
    let mut last = builder.pop().unwrap_or_default();
    if let Some(new) = last.push(&self, level) {
      builder.push(last);
      builder.push(new);
    } else {
      builder.push(last);
    }

    if !self.sections.is_empty() {
      for section in &self.sections {
        builder = section.build(builder, level + 1);
      }
    }
    builder
  }

  fn text(&self, wiki: &Wiki) -> Lemma {
    let mut lemma = Lemma::default();
    let section = self.name.general_species().unwrap_or(SectionSpecies::Unknown);
    for text in &self.content {
      text.text(&mut lemma, &section, &wiki);
    }
    lemma
  }
}

#[derive(Debug, Clone)]
enum Piece {
  Raw(String),
  Template(Params),
}

impl Piece {
  fn text(&self, prefix: &str, lemma: &mut Lemma, suffix: &str, section: &SectionSpecies, wiki: &Wiki) {
    match self {
      Self::Raw(raw) => if !raw.is_empty() { lemma.append_value(prefix, raw, suffix); },
      Self::Template(map) => {
        if let Some(template) = TEMPLATES.get(&map.com) {
          let mut map = map.clone();
          map.section = *section;
          let mut com = match std::process::Command::new(template)
            .env("ENV_MAINWORD", &wiki.word)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
          {
            Err(e) => {
              eprint!("template {} failed: {}", map.com, e);
              return;
            },
            Ok(v) => v,
          };
          let stdin = com.stdin.take().unwrap();
          let stdout = com.stdout.take().unwrap();
          let mut stderr = com.stderr.take().unwrap();
          serde_json::to_writer(stdin, &map).unwrap();
          let com = com.wait().unwrap();
          let mut err = String::new();
          let _ = stderr.read_to_string(&mut err).map_err(|e| eprintln!("bad stderr: {}", e));
          if !err.is_empty() { eprint!("template {}: {}", map.com, err); }
          if !com.success() { eprintln!("template {} failed with {:?}", map.com, com.code()); return; }

          if let Ok(new_lemma) = serde_json::from_reader(stdout).map_err(|e| eprintln!("bad json from {}: {}", map.com, e)) {
            *lemma += new_lemma;
          }
        } else {
          eprintln!("unknown template: {}", map.com);
        }
      },
    }
  }
}

#[derive(Debug, Clone)]
enum Text {
  Text(Vec<Piece>),
  List(u8, Vec<Piece>),
}

impl Text {
  named!(template_open<&str, &str, WikiError<&str>>, tag!("{{"));
  named!(template_separator<&str, &str, WikiError<&str>>, tag!("|"));
  named!(template_close<&str, &str, WikiError<&str>>, tag!("}}"));
  named!(link_open<&str, &str, WikiError<&str>>, tag!("[["));
  named!(link_close<&str, &str, WikiError<&str>>, tag!("]]"));
  named!(external_link_open<&str, &str, WikiError<&str>>, tag!("["));
  named!(external_link_close<&str, &str, WikiError<&str>>, tag!("]"));
  named!(template<&str, (Vec<Option<String>>, Vec<Vec<String>>), WikiError<&str>>,
    map_res!(
      delimited!(
        Self::template_open,
        |input: &str| -> IResult<&str, &str, WikiError<&str>> {
          let mut q = 2;
          let mut end = 0;
          for c in input.chars() {
            if c == '{' { q += 1; }
            else if c == '}' { q -= 1; }
            end += c.len_utf8();
            if q == 0 {
              end -= 2;
              let data = &input[0..end];
              let tail = &input[end..];
              return Ok((tail, data));
            }
          }
          Err(nom::Err::Error(WikiError::OpenNotMatchesClose))
        },
        Self::template_close
      ),
      Self::template_parser
    )
  );
  fn template_parser<'a>(s: &'a str) -> Result<(Vec<Option<String>>, Vec<Vec<String>>), WikiError<&'a str>> {
    let mut input = s;
    let mut v = Vec::new();
    let mut subv = Vec::new();
    let mut multi = 0;
    let mut headers = Vec::new();
    let mut header = None;
    let mut text = String::new();
    while let Some(c) = input.chars().next() {
      input = &input[c.len_utf8()..];
      match c {
        '=' => {
          header = Some(text);
          text = String::new();
        },
        '|' => {
          headers.push(header.take());
          subv.push(text);
          text = String::new();
          v.push(subv);
          subv = Vec::new();
        },
        '(' => {
          multi += 1;
        },
        ')' => {
          multi -= 1;
        },
        '[' => {
          let (tail, (link, alter)) = Self::link(input).or_else(|_| Self::external_link(input)).map_err(|_| WikiError::BadTemplate)?;
          if let Some(alter) = alter {
    //        subs.insert(link.to_owned());
            text += alter;
          } else {
            text += link;
          }
          input = tail;
        },
        ',' if multi == 2 => {
          subv.push(text);
          text = String::new();
        },
        c => {
          text.push(c);
        },
      }
    }
    headers.push(header);
    subv.push(text);
    v.push(subv);
    Ok((headers, v))
  }
  fn any_link(input: &str) -> IResult<&str, (&str, Option<&str>), WikiError<&str>> {
    let mut end = 0;
    let mut word_end = None;
    for c in input.chars() {
      if c == '|' {
        word_end = Some(end);
      } else if c == ']' {
        let tail = &input[end..];
        return Ok((tail, if let Some(word_end) = word_end {
          let word = &input[0..word_end];
          let alter = &input[word_end+1..end];
          (word, Some(alter))
        } else {
          let word = &input[0..end];
          (word, None)
        }));
      }
      end += c.len_utf8();
    }
    Err(nom::Err::Error(WikiError::OpenNotMatchesClose))
  }
  named!(link<&str, (&str, Option<&str>), WikiError<&str>>, delimited!(Self::link_open, Self::any_link, Self::link_close));
  named!(external_link<&str, (&str, Option<&str>), WikiError<&str>>, delimited!(Self::external_link_open, Self::any_link, Self::external_link_close));
  named!(wrapped_template<&str, Piece, WikiError<&str>>, map!(Self::template, |(headers, values)| {
    let mut id = 0;
    let mut data: HashMap<String, Vec<String>> = headers.into_iter().zip(values.into_iter()).map(|(header, values)| {
      let header = header.map(|p| p.to_owned()).unwrap_or_else(|| {
        let out = format!("{}", id);
        id += 1;
        out
      });
      (header, values.into_iter().map(|v| v.to_owned()).collect())
    }).collect();

    let com = data.remove("0").map(|v| v[0].clone()).unwrap_or_default();
    Piece::Template(Params {
      section: SectionSpecies::Unknown,
      com,
      args: data,
    })
  }));
  named!(list<&str, usize, WikiError<&str>>, map!(take_while1!(|c| c == '#' || c == '*' || c == ':'), |r| r.len())); // it works only in the beginning

  fn parse<'a>(mut input: &'a str, subs: &mut HashSet<String>) -> IResult<&'a str, Self, WikiError<&'a str>> {
    let list = Self::list(input).map(|(tail, deep)| {
      input = tail;
      deep
    }).ok();

    let mut data = String::new();
    let mut pieces = Vec::new();

    while !input.is_empty() {
      if let Ok((tail, mut template)) = Self::wrapped_template(input) {
        if let Piece::Template(template) = &mut template {
          println!("args: {:?}", template.args);
          for parts in template.args.values_mut() {
            println!("parts: {:?}", parts);
            for part in parts.iter_mut() {
              println!("part: {:?}", part);
              let mut split = part.split('|');
              let sub = split.next().unwrap();
              if let Some(form) = split.next() {
                println!("Z: \x1b[35m{}\x1b[0m", sub);
                subs.insert(sub.to_string());
                *part = form.to_string();
              }
            }
          }
        }
        if !data.is_empty() {
          pieces.push(Piece::Raw(data));
          data = String::new();
        }
        pieces.push(template);
        input = tail;
      } else {
        if let Ok((tail, (link, alter))) = Self::link(input) {
          if let Some(alter) = alter {
            subs.insert(link.to_owned());
            data += alter;
          } else {
            data += link;
          }
          input = tail;
        } else if let Ok((tail, _link)) = Self::external_link(input) {
          input = tail;
        } else {
          let mut chars = input.chars();
          data.push(chars.next().unwrap());
          input = chars.as_str();
        }
      }
    }
    if !data.is_empty() {
      pieces.push(Piece::Raw(data));
    }

    let text = if let Some(list) = list {
      Self::List(list as _, pieces)
    } else {
      Self::Text(pieces)
    };

    Ok((input, text))
  }

  fn text(&self, lemma: &mut Lemma, section: &SectionSpecies, wiki: &Wiki) {
    match self {
      Self::Text(texts) => {
        for text in texts {
          text.text("", lemma, "", section, wiki);
        }
      },
      Self::List(level, texts) => {
        let mut prefix = String::new();
        for _ in 0..*level { prefix += "*"; }
        prefix.push(' ');
        for text in texts {
          text.text(&prefix, lemma, "\n", section, wiki);
        }
      },
    }
  }
}

#[derive(Debug)]
struct Template {
  code: String,
  attributes: Vec<String>,
}

fn main() {
  let arg = std::env::args().nth(1).unwrap();
  let languages = vec![
    "Latin".to_owned(),
    "French".to_owned(),
    "Italian".to_owned(),
    "English".to_owned(),
    "German".to_owned(),
  ];
  scan(&arg, &languages);
}

fn scan(page: &str, languages: &[String]) {
  let mut wiki = Wiki::new(page, languages);
  let bases = Bases::new().unwrap();
  loop {
    match wiki.parse(&bases, 1) {
      Ok(false) => {
        break;
      },
      Ok(true) => {},
      Err(e) => {
        eprintln!("{}", e);
      }
    }
  }
}

#[derive(Clone, Debug, Default)]
struct Word {
  sections: [Option<(WordSection, usize)>; 11],
}

impl Word {
  fn push(&mut self, section: &WordSection, level: usize) -> Option<Self> {
    let target = section.name.species()?;
    let mut push = section.clone();
    push.sections.clear();
    if self.sections[target].is_some() {
      let mut clone = self.clone();
      clone.sections[target] = Some((section.clone(), level));
      for section in &mut clone.sections {
        if let Some((test, lvl)) = section.take() {
          if lvl <= level {
            *section = Some((test, lvl));
          }
        }
      }
      Some(clone)
    } else {
      self.sections[target] = Some((section.clone(), level));
      None
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Remote(#[from] remote::Error),
  #[error(transparent)]
  Database(#[from] database::Error),
}

// {} — hide from translation
// [] — hide in tests
// _x_ — this word

/* ToDo:
  (has sample) → sample without _{}_ = question
  insert form even if no value
  get subs from template ≈ 741
  template in template
*/
