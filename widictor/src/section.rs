use crate::wiki_error::WikiError;

#[derive(Debug, PartialEq, std::cmp::Eq, std::hash::Hash, Clone, Copy)]
pub enum Section {
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
  pub(crate) fn species(&self) -> Option<usize> {
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

  /*
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
  */
}

impl From<&str> for Section {
  fn from(value: &str) -> Self {
    let mut list = [
      ("Declension", Section::Declension),
      ("Derived terms", Section::DerivedTerms),
      ("Related terms", Section::RelatedTerms),
      ("Descendants", Section::Descendants),
      ("See also", Section::SeeAlso),
      ("Etymology", Section::Etymology),
      ("Pronunciation", Section::Pronunciation),
      ("References", Section::References),
      ("Further reading", Section::FurtherReading),
      ("Alternative forms", Section::AlternativeForms),
      ("Conjugation", Section::Conjugation),
      ("Usage notes", Section::UsageNotes),
      ("Translations", Section::Translations),
      ("Anagrams", Section::Anagrams),
      ("Conjunction", Section::Conjunction),
      ("Synonyms", Section::Synonyms),
      ("Antonyms", Section::Antonyms),
      ("Determiner", Section::Determiner),
      ("Contraction", Section::Contraction),
      ("Inflection", Section::Inflection),
      ("Compounds", Section::Compounds),

      ("Noun", Section::Noun),
      ("Proper noun", Section::Noun),
      ("Verb", Section::Verb),
      ("Adjective", Section::Adjective),
      ("Preposition", Section::Preposition),
      ("Pronoun", Section::Pronoun),
      ("Participle", Section::Participle),
      ("Interjection", Section::Interjection),
      ("Adverb", Section::Adverb),
      ("Numeral", Section::Numeral),
      ("Particle", Section::Particle),
    ].iter().map(|(template, value)|
      move |test: &str| if test.starts_with(template) { Some(value.clone()) } else { None }
    );
    while let Some(l) = list.next() {
      if let Some(value) = l(value) { return value; }
    }
    Self::Unknown
  }
}

#[cfg(test)]
#[test]
fn test() {
  assert_eq!(Section::Noun, Section::from("Noun"));
  assert_eq!(Section::Unknown, Section::from("U"));
}
