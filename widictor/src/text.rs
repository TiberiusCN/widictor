use std::collections::{HashMap, HashSet};
use nom::*;
use crate::wiki_error::WikiError;
use crate::substr::SubStr;

mod template;
pub use template::Template;

#[derive(Debug, Clone)]
pub enum Text {
  Raw(String),
  Tab(u8),
  Template(Template),
}

impl Text {
  named!(link_open<&str, &str, WikiError<&str>>, tag!("[["));
  named!(link_close<&str, &str, WikiError<&str>>, tag!("]]"));
  named!(external_link_open<&str, &str, WikiError<&str>>, tag!("["));
  named!(external_link_close<&str, &str, WikiError<&str>>, tag!("]"));
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
  named!(list<&str, usize, WikiError<&str>>, map!(take_while1!(|c| c == '#' || c == '*' || c == ':'), |r| r.len())); // it works only in the beginning

  pub fn parse<'a>(mut input: &'a str, subs: &mut HashSet<String>) -> IResult<&'a str, Self, WikiError<&'a str>> {
    let list = Self::list(input).map(|(tail, deep)| {
      input = tail;
      deep
    }).ok();

    let (input, pieces) = Self::parse_text(input, subs)?;

    let text = if let Some(list) = list {
      Self::List(list as _, pieces)
    } else {
      Self::Text(pieces)
    };

    Ok((input, text))
  }
  fn parse_text<'a>(mut input: &'a str, subs: &mut HashSet<String>) -> IResult<(), Vec<Piece>, WikiError<&'a str>> {
    let mut data = String::new();
    let mut pieces = Vec::new();

    while !input.is_empty() {
      if let Ok((tail, (template, sub))) = Self::template(input) {
        for sub in sub {
          subs.insert(sub);
        }
        /*
        if let Piece::Template(template) = &mut template {
          for parts in template.args.values_mut() {
            for part in parts.iter_mut() {
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
        */
        /*
        if !data.is_empty() {
          pieces.push(Piece::Raw(data));
          data = String::new();
        }
        */
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

    Ok(((), pieces))
  }

  /*
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
  */
}
