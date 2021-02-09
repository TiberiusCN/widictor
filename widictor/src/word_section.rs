use std::{iter::FromIterator, rc::Rc};

use nom::*;

use crate::section::Section;
use crate::wiki_error::WikiError;

#[derive(Debug, Clone, PartialEq)]
struct WordSection {
  name: Section,
  level: usize,
}

/*
#[derive(Debug, Clone)]
struct WordSectionTree {
  section: WordSection,
  sections: Vec<WordSection>,
  content: Vec<Text>,
}
*/

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

  fn parse(input: &str) -> IResult<(), Self, WikiError<&str>> {
    let value = Self::word_section(input)?;
    let tail = value.0;
    if !tail.is_empty() { return Err(WikiError::UnexpectedTail(tail).into()); }
    let (value, level) = value.1;
    let section = Section::from(value);

    Ok(((), Self {
      name: section,
      level: level - 1,
    }))
  }
}

impl Conflictable for WordSection {
  fn conflict(&self, parent: &Self) -> bool {
    self.level > parent.level || (self.level == parent.level && self.name.species() == parent.name.species())
  }
}

#[cfg(test)]
#[test]
fn test_conflict() {
  [
    "===Pronunciation===",
      "====Noun 1====",
        "=====Conjugation 1=====",
      "====Noun 2====",
      "====Verb====",
    "===Etymology===",
  ].iter()
    .map(|s| WordSection::parse(s).unwrap().1)
    .collect::<Vec<_>>().as_slice().windows(2)
    .zip([false, false, true, true, true, true].iter().copied())
    .for_each(|(secs, conflict)| if secs[0].conflict(&secs[1]) != conflict {
      let val = if conflict { "≠" } else { "=" };
      panic!("{:?} \x1b[31m{}\x1b[0m {:?}", secs[0], val, secs[1]);
    });
}

#[cfg(test)]
#[test]
fn test_parse_section() {
  [
    ("===Noun===", Some((0, Section::Noun))),
    ("==== ====", Some((1, Section::Unknown))),
    ("===== =====", Some((2, Section::Unknown))),
    ("==Noun==", None),
    ("Noun", None),
    ("====Noun==", None),
    ("===Noun====", None),
  ]
    .iter()
    .map(|(q, a)| (WordSection::parse(q).ok().map(|v| v.1), a.map(|a| WordSection { level: a.0, name: a.1 })))
    .for_each(|(q, a)| assert_eq!(q, a));
}

trait Conflictable {
  fn conflict(&self, other: &Self) -> bool;
}

enum Seq<T> {
  None,
  Rc(Rc<T>, Rc<Seq<T>>),
}

impl<T> Default for Seq<T> {
  fn default() -> Self {
    Self::None
  }
}

impl<T> Seq<T> {
  fn insert(self: Rc<Self>, value: T) -> Rc<Self> {
    Self::rc(value, self)
  }
  fn rc(value: T, root: Rc<Self>) -> Rc<Self> {
    Rc::new(Self::Rc(Rc::new(value), root))
  }
  fn into_iter(self: Rc<Self>) -> SeqIter<T> {
    SeqIter(self)
  }
}

struct SeqIter<T>(Rc<Seq<T>>);

impl<T> Iterator for SeqIter<T> {
  type Item = Rc<T>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.0.clone().as_ref() {
      Seq::None => None,
      Seq::Rc(t, b) => {
        self.0 = b.clone();
        Some(t.clone())
      }
    }
  }
}

impl<T: Conflictable> Seq<T> {
  // :(
  fn graft(self: Rc<Self>, value: T) -> (Rc<Self>, Option<Rc<Self>>) {
    match self.as_ref() {
      Self::None => (Self::rc(value, self), None),
      Self::Rc(me, branch) => {
        if me.conflict(&value) {
          (branch.clone().graft(value).0, Some(self.clone()))
        } else {
          (self.insert(value), None)
        }
      },
    }
  }
}

struct Tree<T: Conflictable>(Vec<Rc<Seq<T>>>);

impl<T: Conflictable> Default for Tree<T> {
  fn default() -> Self { Self(Vec::new()) }
}

impl<T: Conflictable> Tree<T> {
  fn new() -> Self {
    Self::default()
  }
  fn graft(mut self, value: T) -> Self {
    let root = self.0.pop().unwrap_or_default();
    let (value, branch) = root.graft(value);
    if let Some(value) = branch {
      self.0.push(value);
    }
    self.0.push(value);
    self
  }
  fn into_iter(mut self) -> impl Iterator<Item = Rc<Seq<T>>> {
    self.0.into_iter()
  }
}

impl<T: Conflictable> FromIterator<T> for Tree<T> {
  fn from_iter<Q: IntoIterator<Item = T>>(iter: Q) -> Self {
    iter.into_iter().fold(Self::new(), |mut acc, sec| acc.graft(sec))
  }
}

#[cfg(test)]
#[test]
fn test_seq() {
  let tree = [
    "===Etymology 1===",
     "====Pronunciation====",
     "====Noun 1====",
      "=====Conjugation=====",
      "=====Synonyms=====", // Et Pr N C S
     "====Noun 2====",
      "=====Conjugation=====", // Et Pr N C
     "====Verb====", // Et Pr V
    "===Etymology 2===",
     "====Noun====", // Et N
  ].iter()
    .map(|s| WordSection::parse(s).unwrap().1)
    .filter(|s| s.name.species().is_some())
    .collect::<Tree<_>>();

  let untree: Vec<(usize, Section)> = tree.into_iter().enumerate().flat_map(|(bid, branch)| branch.into_iter().map(move |v| (bid, v.name))).collect();
  let target = vec![
    (0, Section::Synonyms),
    (0, Section::Conjugation),
    (0, Section::Noun),
    (0, Section::Pronunciation),
    (0, Section::Etymology),
    (1, Section::Conjugation),
    (1, Section::Noun),
    (1, Section::Pronunciation),
    (1, Section::Etymology),
    (2, Section::Verb),
    (2, Section::Pronunciation),
    (2, Section::Etymology),
    (3, Section::Noun),
    (3, Section::Etymology),
    ];
  let mut template = target.into_iter();
  let mut got = untree.into_iter();
  loop {
    match (template.next(), got.next()) {
      (None, None) => break,
      (None, Some(x)) => panic!("unexpected branch: {:?}", x),
      (Some(x), None) => panic!("lost branch: {:?}", x),
      (Some(a), Some(b)) => assert_eq!(a, b),
    }
  }
}

/*
{
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
*/
