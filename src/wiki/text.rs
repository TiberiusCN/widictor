#[allow(unused)]
use crate::wiki as m;
use m::wiki_error::WikiError;

use nom::{
  bytes::complete::{is_not, tag, take_while1},
  combinator::map,
  sequence::delimited,
  IResult,
};
use std::collections::HashSet;
pub use template::Template;

mod template;

#[derive(Debug, Clone)]
pub enum Text {
  Raw(String),
  Tab(u8),
  Template(Template),
}

impl Text {
  // TODO: quotation, trim
  fn link_open(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(tag("[[")(src)?)
  }
  fn link_close(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(tag("]]")(src)?)
  }
  fn external_link_open(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(tag("[")(src)?)
  }
  fn external_link_close(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(tag("]")(src)?)
  }
  fn any_link(input: &str) -> IResult<&str, (&str, Option<&str>), WikiError<&str>> {
    let mut end = 0;
    let mut word_end = None;
    for c in input.chars() {
      if c == '|' {
        word_end = Some(end);
      } else if c == ']' {
        let tail = &input[end..];
        return Ok((
          tail,
          if let Some(word_end) = word_end {
            let word = &input[0..word_end];
            let alter = &input[word_end + 1..end];
            (word, Some(alter))
          } else {
            let word = &input[0..end];
            (word, None)
          },
        ));
      }
      end += c.len_utf8();
    }
    Err(nom::Err::Error(WikiError::OpenNotMatchesClose))
  }
  fn link(src: &str) -> IResult<&str, (&str, Option<&str>), WikiError<&str>> {
    Ok(delimited(Self::link_open, Self::any_link, Self::link_close)(src)?)
  }
  fn external_link(src: &str) -> IResult<&str, (&str, Option<&str>), WikiError<&str>> {
    Ok(delimited(Self::external_link_open, Self::any_link, Self::external_link_close)(src)?)
  }
  fn list(src: &str) -> IResult<&str, usize, WikiError<&str>> {
    Ok(map(take_while1(|c| c == '#' || c == '*' || c == ':'), |r: &str| r.len())(src)?)
  }
  fn raw(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(is_not("{}[]")(src)?)
  }
  pub fn parse<'a>(input: &'a str, subs: &mut HashSet<String>) -> IResult<&'a str, Self, WikiError<&'a str>> {
    let mut err_chain = String::new();
    let out =
      if let Ok((s, list)) = Self::list(input).map_err(|e| err_chain += format!("→ test list: {:?}\n", e).as_str()) {
        (s, Self::Tab(list as _))
      } else if let Ok((s, (link, url))) =
        Self::link(input).map_err(|e| err_chain += format!("→ test link: {:?}\n", e).as_str())
      {
        subs.insert(url.unwrap_or(link).to_owned());
        (s, Self::Raw(link.to_owned()))
      } else if let Ok((s, (link, _url))) =
        Self::external_link(input).map_err(|e| err_chain += format!("→ test elink: {:?}\n", e).as_str())
      {
        (s, Self::Raw(link.to_owned()))
      } else if let Ok((s, template)) =
        Template::parse(input, subs).map_err(|e| err_chain += format!("→ test template: {:?}\n", e).as_str())
      {
        (s, Self::Template(template))
      } else {
        // println!("\x1b[31m«{}»\x1b[0m as raw:{:?}", input, Self::raw(input));
        let (s, raw) = Self::raw(input).map_err(|e| {
          err_chain += format!("→ test raw: {:?}\n", e).as_str();
          eprintln!("{}", err_chain);
          e
        })?;
        (s, Self::Raw(raw.to_owned()))
      };
    Ok(out)
  }
}
