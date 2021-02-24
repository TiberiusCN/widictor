use std::rc::Rc;

use nom::*;
use crate::{wiki_error::WikiError, word_section::{Tree, WordSection}};

#[derive(Debug)]
pub struct Language<T, S: std::convert::Into<Rc<WordSection<T>>>> {
  pub name: String,
  pub sections: Vec<S>,
  phantom: std::marker::PhantomData<T>,
}

impl Language<(), WordSection<()>> {
  named!(language<&str, &str, WikiError<&str>>, delimited!(tag!("=="), take_while1!(|c: char| c.is_alphabetic() || c.is_whitespace()), tag!("==")));

  pub fn parse(input: &str) -> IResult<&str, Self, WikiError<&str>> {
    let value = Self::language(input)?;

    Ok((value.0, Self {
      name: value.1.to_string(),
      sections: vec![WordSection::empty()],
      phantom: Default::default(),
    }))
  }
}

impl<T> Language<T, WordSection<T>> {
  pub fn section(&mut self) -> &mut WordSection<T> {
    self.sections.last_mut().unwrap()
  }
  pub fn convert<N, TtoN: FnMut(T) -> N>(self, mut conv: TtoN) -> Language<N, WordSection<N>> {
    Language {
      name: self.name,
      sections: self.sections.into_iter().map(|v| v.convert(&mut conv)).collect(),
      phantom: Default::default(),
    }
  }
  pub fn try_convert<N, E, TtoN: FnMut(T) -> Result<N, E>>(self, mut conv: TtoN) -> Result<Language<N, WordSection<N>>, E> {
    Ok(Language {
      name: self.name,
      sections: self.sections.into_iter().map(|v| v.try_convert(&mut conv)).collect::<Result<_, E>>()?,
      phantom: Default::default(),
    })
  }
  pub fn fold_convert<N, TtoN: FnMut(Vec<N>, T) -> Vec<N>>(self, mut conv: TtoN) -> Language<N, WordSection<N>> {
    Language {
      name: self.name,
      sections: self.sections.into_iter().map(|v| v.fold_convert(&mut conv)).collect(),
      phantom: Default::default(),
    }
  }
  pub fn subdivide(self) -> Vec<Language<T, Rc<WordSection<T>>>> {
    let name = self.name;
    let tree = self.sections.into_iter().filter(|s| s.name.species().is_some()).collect::<Tree<_>>();
    tree.into_iter().map(|v| Language {
      name: name.clone(),
      sections: v.into_iter().collect(),
      phantom: Default::default(),
    }).collect()
  }
}
