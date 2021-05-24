use std::collections::{HashMap, HashSet};
use nom::{IResult, bytes::complete::tag, sequence::delimited};

use crate::wiki_error::WikiError;
use super::Text;

#[derive(Debug, Clone)]
pub struct Template {
  pub com: Vec<Text>,
  pub args: HashMap<String, (Option<Vec<Text>>, Vec<Text>)>,
}

enum ArgType<'a> {
  Auto,
  Force(&'a str),
}

impl Template {
  fn open(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(tag("{{")(src)?)
  }
  fn separator(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(tag("|")(src)?)
  }
  fn close(src: &str) -> IResult<&str, &str, WikiError<&str>> {
    Ok(tag("}}")(src)?)
  }
  fn template(src: &str) -> IResult<&str, Vec<&str>, WikiError<&str>> {
    Ok(delimited(
      Self::open,
      Self::template_parser,
      Self::close
    )(src)?)
  }
  fn arg(s: &str) -> IResult<&str, &str, WikiError<&str>> {
    let mut br = 0;
    let mut length = 0;
    for c in s.chars() {
      match c {
        '|' | '}' if br == 0 => break,
        '{' => br += 1,
        '}' => br -= 1,
        _ => {},
      }
      length += c.len_utf8();
    }
    let (head, tail) = s.split_at(length);
    Ok((tail, head))
  }
  fn template_parser(mut s: &str) -> IResult<&str, Vec<&str>, WikiError<&str>> {
    let mut out = Vec::new();
    loop {
      let res = Self::arg(s)?;
      s = res.0;
      out.push(res.1);
      if Self::close(s).is_ok() {
        return Ok((s, out));
      }
      s = Self::separator(s)?.0;
    }
  }
  pub fn parse<'a>(s: &'a str, subs: &mut HashSet<String>) -> IResult<&'a str, Self, WikiError<&'a str>> {
    let (tail, args) = Self::template(s)?;
    // unwrapped:
    //   ((X,Y)) → alt
    //   X<…> → params
    let args = args.into_iter();
    let mut unordered = Vec::new();
    let mut params = HashMap::with_capacity(args.len());
    for v in args {
      println!("ARG: {}", &v);
      let (name, v) = if let Some(split) = v.find('=') {
        let out = v.split_at(split);
        (ArgType::Force(out.0), out.1)
      } else {
        (ArgType::Auto, v)
      };
      let mut value = Vec::new();
      let mut v = v;
      while !v.is_empty() {
        let (tail, s) = Text::parse(v, subs)?;
        v = tail;
        value.push(s);
      }
      match name {
        ArgType::Auto => {
          unordered.push(value);
        },
        ArgType::Force(name) => {
          params.insert(name.to_string(), value);
        },
      }
    }
    let mut id = 0;
    for value in unordered {
      let mut name = format!("{}", id);
      while params.get(&name).is_some() {
        id += 1;
        name = format!("{}", id);
      }
      params.insert(name, value);
    }
    let header = params.remove("0").ok_or(WikiError::BadTemplate)?;

    Ok((tail, Template {
      com: header,
      args: params,
    }))
  }
}
