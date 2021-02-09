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
