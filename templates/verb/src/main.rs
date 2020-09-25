use template::*;
use serde_json::to_writer;
use std::env::var as env_var;
use std::collections::{HashSet, HashMap};

fn main() {
  let word = env_var("ENV_MAINWORD").unwrap();
  let lang = env_var("ENV_0").unwrap();

  let data = match lang.as_str() {
    "la-verb" => latina(&word),
    u => panic!("unsupported: {}", u),
  };

  to_writer(std::io::stdout(), &data).unwrap();
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
enum Rule {
  Set,
  Allow,
  Forbidden,
  Auto,
}
impl Default for Rule {
  fn default() -> Self { Self::Auto }
}
impl From<Rule> for bool {
  fn from(src: Rule) -> Self {
    src == Rule::Set
  }
}

#[derive(Debug, Clone)]
struct Conjugation {
  conj: ConjugationType,

  lemma: String,
  iv: bool, // + false, ++ true
  deponent: bool,
  semi_deponent: bool,

  perfect_stem: Option<String>,
  supine_stem: Option<String>,

  // ToDo: no pass links in parameters in widictor
  prefix: Option<String>, // all if !passive_prefix
  passive_prefix: Option<String>,
  suffix: Option<String>, // all if !passive_suffix
  passive_suffix: Option<String>,

  // todo: table parts?
}

#[derive(Debug, Clone)]
enum ConjugationType {
  C1, C2, C3, C4,
  Irreg(Vec<String>),
}

impl Declension {
  fn encode(&self) -> (String, HashSet<String>) {
    let mut tags = HashSet::new();
    
    (match self {
      Self::D1 { abus, greek, ma, me, loc, am, plural, single } => {
        tags.insert("declension_1".to_owned());
        if *abus { tags.insert("d1_abus".to_owned()); }
        if *greek { tags.insert("greek".to_owned()); }
        if *ma { tags.insert("d1_ma".to_owned()); }
        if *me { tags.insert("d1_me".to_owned()); }
        if *loc { tags.insert("loc".to_owned()); }
        if *am { tags.insert("d1_am".to_owned()); }
        "1"
      },
      Self::D2 { neuter, greek, ius, er, voci, ium, loc, us, plural, single } => {
        tags.insert("declension_2".to_owned());
        if *ius { tags.insert("d2_ius".to_owned()); }
        if *er { tags.insert("d2_er".to_owned()); }
        if *voci { tags.insert("d2_voci".to_owned()); }
        if *ium { tags.insert("d2_ium".to_owned()); }
        if *us { tags.insert("d2_us".to_owned()); }
        if *loc { tags.insert("loc".to_owned()); }
        if *neuter { tags.insert("neuter".to_owned()); }
        if *greek { tags.insert("greek".to_owned()); }
        "2"
      },
      Self::D3 { neuter, i, pure, ignis, navis, loc, plural, acc_im, acc_im_in, acc_im_in_em, acc_im_em, acc_im_occ_em, acc_em_im, abl_i, abl_i_e, abl_e_i, abl_e_occ_i, single } => {
        tags.insert("declension_3".to_owned());
        if *i { tags.insert("d3_i".to_owned()); }
        if *pure { tags.insert("d3_pure".to_owned()); }
        if *ignis { tags.insert("d3_ignis".to_owned()); }
        if *navis { tags.insert("d3_navis".to_owned()); }
        if *acc_im { tags.insert("d3_acc_im".to_owned()); }
        if *acc_im_in { tags.insert("d3_acc_im_in".to_owned()); }
        if *acc_im_in_em { tags.insert("d3_acc_im_in_em".to_owned()); }
        if *acc_im_em { tags.insert("d3_acc_im_em".to_owned()); }
        if *acc_im_occ_em { tags.insert("d3_acc_im_occ_em".to_owned()); }
        if *acc_em_im { tags.insert("d3_acc_em_im".to_owned()); }
        if *abl_i { tags.insert("d3_abl_i".to_owned()); }
        if *abl_i_e { tags.insert("d3_abl_i_e".to_owned()); }
        if *abl_e_i { tags.insert("d3_abl_e_i".to_owned()); }
        if *abl_e_occ_i { tags.insert("d3_abl_e_occ_i".to_owned()); }
        if *neuter { tags.insert("neuter".to_owned()); }
        if *loc { tags.insert("loc".to_owned()); }
        if *plural { tags.insert("plural".to_owned()); }
        "3"
      },
      Self::D4 { neuter, greek, plural, single } => {
        tags.insert("declension_4".to_owned());
        if *neuter { tags.insert("neuter".to_owned()); }
        if *greek { tags.insert("greek".to_owned()); }
        "4"
      },
      Self::D5 { ies, plural, single } => {
        tags.insert("declension_5".to_owned());
        if *ies { tags.insert("d5_ies".to_owned()); }
        "5"
      },
      Self::Indecl {} => {
        "indecl"
      },
      Self::Irreg {} => {
        "irreg"
      },
    }.to_owned(), tags)
    
  }

  fn d1() -> Self {
    Self::D1 {
      abus: false,
      greek: false,
      ma: false,
      me: false,
      loc: false,
      am: false,
      plural: false,
      single: false,
    }
  }

  fn d2() -> Self {
    Self::D2 {
      neuter: false,
      er: false,
      greek: false,
      ius: false,
      voci: false,
      ium: false,
      loc: false,
      us: false,
      plural: false,
      single: false,
    }
  }
  
  fn d3() -> Self {
    Self::D3 {
      neuter: false,
      i: false,
      pure: false,
      ignis: false,
      navis: false,
      loc: false,
      plural: false,
      acc_im: false,
      acc_im_in: false,
      acc_im_in_em: false,
      acc_im_em: false,
      acc_im_occ_em: false,
      acc_em_im: false,
      abl_i: false,
      abl_i_e: false,
      abl_e_i: false,
      abl_e_occ_i: false,
      single: false,
    }
  }

  fn d4() -> Self {
    Self::D4 {
      neuter: false,
      greek: false,
      plural: false,
      single: false,
    }
  }

  fn d5() -> Self {
    Self::D5 {
      ies: false,
      plural: false,
      single: false,
    }
  }

  fn table(&self, lemma: &str, stem: &str) -> HashMap<Case, String> {
    let mut map = HashMap::new();
    let from_stem_or_lemma = |endings: Vec<&str>| -> String {
      if endings.is_empty() {
        lemma.to_owned()
      } else {
        let mut s = String::new();
        for ending in endings {
          s += &format!("{}{}/", stem, ending);
        }
        s.pop();
        s
      }
    };

    match self {
      Self::D1 { abus, greek, ma, me, loc, am, plural, single } => {
        if !*plural {
          map.insert(Case::GenSg, from_stem_or_lemma(match
            greek {
              false => vec!["ae"],
              true  => vec!["ēs"],
          }));
          map.insert(Case::DatSg, from_stem_or_lemma(vec!["ae"])); 
          map.insert(Case::AccSg, from_stem_or_lemma(match
            (  greek, ma,    me,    am) {
              (false, false, false, false) => vec!["am"],
              (true,  false, false, false) => vec!["ēn"],
              (true,  true,  false, false) => vec!["ān"],
              (true,  false, true,  false) => vec!["ēn"],
              (false, false, false, true ) => vec!["ām"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::AblSg, from_stem_or_lemma(match
            (  greek, me) {
              (false, false) => vec!["ā"],
              (true,  false) => vec!["ē"],
              (true,  true ) => vec!["ē"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::VocSg, from_stem_or_lemma(match
            (  greek, ma,    me,    am) {
              (false, false, false, false) => vec!["a"],
              (true,  false, false, false) => vec!["ē"],
              (true,  true,  false, false) => vec!["ā"],
              (true,  false, true,  false) => vec!["ē"],
              (true,  false, false, true ) => vec!["ām"],
              _ => panic!("bad subtypes"),
          }));
          if *loc {
            map.insert(Case::LocSg, from_stem_or_lemma(vec!["ae"]));
          }
        }
        if !*single {
          map.insert(Case::NomPl, from_stem_or_lemma(vec!["ae"]));
          map.insert(Case::GenPl, from_stem_or_lemma(vec!["arum"]));
          map.insert(Case::DatPl, from_stem_or_lemma(match
            abus {
              true  => vec!["ābus"],
              false => vec!["īs"],
          }));
          map.insert(Case::AccPl, from_stem_or_lemma(vec!["ās"]));
          map.insert(Case::AblPl, from_stem_or_lemma(match
            abus {
              true  => vec!["ābus"],
              false => vec!["īs"],
          }));
          map.insert(Case::VocPl, from_stem_or_lemma(vec!["ae"]));
          if *loc {
            map.insert(Case::LocPl, from_stem_or_lemma(vec!["īs"]));
          }
        }
        if *plural {
          map.insert(Case::NomPl, from_stem_or_lemma(vec![]));
        } else {
          map.insert(Case::NomSg, from_stem_or_lemma(vec![]));
        }
      },
      Self::D2 { neuter, er, greek, ius, voci, ium, loc, us, plural, single } => {
        if !*plural {
          map.insert(Case::GenSg, from_stem_or_lemma(vec!["ī"]));
          map.insert(Case::DatSg, from_stem_or_lemma(vec!["ō"]));
          map.insert(Case::AccSg, from_stem_or_lemma(match
            (  neuter,er,    greek, ius,   voci,  ium,   loc,   us) {
              (false, false, false, false, false, false, false, false) => vec!["um"],
              (true,  false, false, false, false, false, false, false) => vec!["um"],
              (false, true,  false, false, false, false, false, false) => vec!["um"],
              (false, false, true,  false, false, false, false, false) => vec!["on"],
              (true,  false, true,  false, false, false, false, false) => vec!["on"],
              (false, false, false, true,  false, false, false, false) => vec!["um"],
              (false, false, false, true,  true,  false, false, false) => vec!["um"],
              (true,  false, false, false, false, true,  false, false) => vec!["um"],
              (false, false, false, false, false, false, true,  false) => vec!["um"],
              (true,  false, false, false, false, false, true,  false) => vec!["um"],
              (true,  false, false, false, false, false, false, true ) => vec!["us"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::AblSg, from_stem_or_lemma(vec!["ō"]));
          map.insert(Case::VocSg, from_stem_or_lemma(match
            (  neuter,er,    greek, ius,   voci,  ium,   loc,   us) {
              (false, false, false, false, false, false, false, false) => vec!["e"],
              (true,  false, false, false, false, false, false, false) => vec!["um"],
              (false, true,  false, false, false, false, false, false) => vec![],
              (false, false, true,  false, false, false, false, false) => vec!["e"],
              (true,  false, true,  false, false, false, false, false) => vec!["on"],
              (false, false, false, true,  false, false, false, false) => vec!["e"],
              (false, false, false, true,  true,  false, false, false) => vec!["ī"],
              (true,  false, false, false, false, true,  false, false) => vec!["um"],
              (false, false, false, false, false, false, true,  false) => vec!["us"],
              (true,  false, false, false, false, false, true,  false) => vec!["um"],
              (true,  false, false, false, false, false, false, true ) => vec!["us"],
              _ => panic!("bad subtypes"),
          }));
          if *loc {
            map.insert(Case::LocSg, from_stem_or_lemma(vec!["ī"]));
          }
        }
        if !(*neuter && *us) && !*single {
          map.insert(Case::NomPl, from_stem_or_lemma(match
            (  neuter,er,    greek, ius,   voci,  ium,   loc) {
              (false, false, false, false, false, false, false) => vec!["ī"],
              (true,  false, false, false, false, false, false) => vec!["a"],
              (false, true,  false, false, false, false, false) => vec!["ī"],
              (false, false, true,  false, false, false, false) => vec!["ī"],
              (true,  false, true,  false, false, false, false) => vec!["a"],
              (false, false, false, true,  false, false, false) => vec!["ī"],
              (false, false, false, true,  true,  false, false) => vec!["ī"],
              (true,  false, false, false, false, true,  false) => vec!["a"],
              (false, false, false, false, false, false, true ) => vec!["ī"],
              (true,  false, false, false, false, false, true ) => vec!["a"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::GenPl, from_stem_or_lemma(vec!["ōrum"]));
          map.insert(Case::DatPl, from_stem_or_lemma(vec!["īs"]));
          map.insert(Case::AccPl, from_stem_or_lemma(match
            (  neuter,er,    greek, ius,   voci,  ium,   loc) {
              (false, false, false, false, false, false, false) => vec!["ōs"],
              (true,  false, false, false, false, false, false) => vec!["a"],
              (false, true,  false, false, false, false, false) => vec!["ōs"],
              (false, false, true,  false, false, false, false) => vec!["ōs"],
              (true,  false, true,  false, false, false, false) => vec!["a"],
              (false, false, false, true,  false, false, false) => vec!["ōs"],
              (false, false, false, true,  true,  false, false) => vec!["ōs"],
              (true,  false, false, false, false, true,  false) => vec!["a"],
              (false, false, false, false, false, false, true ) => vec!["ōs"],
              (true,  false, false, false, false, false, true ) => vec!["a"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::VocPl, from_stem_or_lemma(match
            (  neuter,er,    greek, ius,   voci,  ium,   loc) {
              (false, false, false, false, false, false, false) => vec!["ī"],
              (true,  false, false, false, false, false, false) => vec!["a"],
              (false, true,  false, false, false, false, false) => vec!["ī"],
              (false, false, true,  false, false, false, false) => vec!["ī"],
              (true,  false, true,  false, false, false, false) => vec!["a"],
              (false, false, false, true,  false, false, false) => vec!["ī"],
              (false, false, false, true,  true,  false, false) => vec!["ī"],
              (true,  false, false, false, false, true,  false) => vec!["a"],
              (false, false, false, false, false, false, true ) => vec!["ī"],
              (true,  false, false, false, false, false, true ) => vec!["a"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::AblPl, from_stem_or_lemma(vec!["īs"]));
          if *loc {
            map.insert(Case::LocPl, from_stem_or_lemma(vec!["īs"]));
          }
        }
        if *plural {
          map.insert(Case::NomPl, from_stem_or_lemma(vec![]));
        } else {
          map.insert(Case::NomSg, from_stem_or_lemma(vec![]));
        }
      },
      Self::D3 { neuter, i, pure, ignis, navis, loc, plural, acc_im, acc_im_in, acc_im_in_em, acc_im_em, acc_im_occ_em, acc_em_im, abl_i, abl_i_e, abl_e_i, abl_e_occ_i, single } => {
        if !*plural {
          map.insert(Case::GenSg, from_stem_or_lemma(vec!["is"]));
          map.insert(Case::DatSg, from_stem_or_lemma(vec!["ī"]));
          map.insert(Case::AccSg, from_stem_or_lemma(match
            (  neuter,i,     pure,  ignis, navis, loc,  acc_im, acc_im_in, acc_im_in_em, acc_im_em, acc_im_occ_em, acc_em_im) {
              (false, false, false, false, false, false,false,  false,     false,        false,     false,         false,     ) => vec!["em"],
              (true,  false, false, false, false, false,false,  false,     false,        false,     false,         false,     ) => vec![],
              (false, true,  false, false, false, false,false,  false,     false,        false,     false,         false,     ) => vec!["em"],
              (true,  true,  false, false, false, false,false,  false,     false,        false,     false,         false,     ) => vec![],
              (true,  true,  true,  false, false, false,false,  false,     false,        false,     false,         false,     ) => vec![],
              (false, true,  false, true,  false, false,false,  false,     false,        false,     false,         false,     ) => vec!["em"],
              (false, true,  false, false, true,  false,false,  false,     false,        false,     false,         false,     ) => vec!["em", "im"],
              (false, false, false, false, false, true ,false,  false,     false,        false,     false,         false,     ) => vec!["em"],
              (_,     true,  _,     _,     _,     _,    true,   false,     false,        false,     false,         false,     ) => vec!["im"],
              (_,     true,  _,     _,     _,     _,    false,  true,      false,        false,     false,         false,     ) => vec!["im", "in"],
              (_,     true,  _,     _,     _,     _,    false,  false,     true,         false,     false,         false,     ) => vec!["im", "in", "em"],
              (_,     true,  _,     _,     _,     _,    false,  false,     false,        true,      false,         false,     ) => vec!["im", "em"],
              (_,     true,  _,     _,     _,     _,    false,  false,     false,        false,     true,          false,     ) => vec!["im", "em"],
              (_,     true,  _,     _,     _,     _,    false,  false,     false,        false,     false,         true,      ) => vec!["em", "im"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::AblSg, from_stem_or_lemma(match
            (  neuter,i,     pure,  ignis, navis, loc,  abl_i, abl_i_e, abl_e_i, abl_e_occ_i) {
              (false, false, false, false, false, false,false, false,   false,   false        ) => vec!["e"],
              (true,  false, false, false, false, false,false, false,   false,   false        ) => vec!["e"],
              (false, true,  false, false, false, false,false, false,   false,   false        ) => vec!["e"],
              (true,  true,  false, false, false, false,false, false,   false,   false        ) => vec!["e"],
              (true,  true,  true,  false, false, false,false, false,   false,   false        ) => vec!["ī"],
              (false, true,  false, true,  false, false,false, false,   false,   false        ) => vec!["ī", "e"],
              (false, true,  false, false, true,  false,false, false,   false,   false        ) => vec!["ī", "e"],
              (false, false, false, false, false, true ,false, false,   false,   false        ) => vec!["e"],
              (_,     _,      _,     _,    _,     _,    true,  false,   false,   false        ) => vec!["ī"],
              (_,     _,      _,     _,    _,     _,    false, true,    false,   false        ) => vec!["ī", "e"],
              (_,     _,      _,     _,    _,     _,    false, false,   true,    false        ) => vec!["e", "ī"],
              (_,     _,      _,     _,    _,     _,    false, false,   false,   true         ) => vec!["e", "ī"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::VocSg, from_stem_or_lemma(vec![]));
          if *loc {
            map.insert(Case::LocSg, from_stem_or_lemma(vec!["ī", "e"]));
          }
        }
        if !*single {
          map.insert(Case::NomPl, from_stem_or_lemma(match
            (  neuter,i,     pure,  ignis, navis, loc,   plural) {
              (false, false, false, false, false, false, false) => vec!["ēs"],
              (true,  false, false, false, false, false, false) => vec!["a"],
              (false, true,  false, false, false, false, false) => vec!["ēs"],
              (true,  true,  false, false, false, false, false) => vec!["a"],
              (true,  true,  true,  false, false, false, false) => vec!["ia"],
              (false, true,  false, true,  false, false, false) => vec!["ēs"],
              (false, true,  false, false, true,  false, false) => vec!["ēs"],
              (false, false, false, false, false, true,  true ) => vec!["es"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::GenPl, from_stem_or_lemma(match
            (  neuter,i,     pure,  ignis, navis, loc,   plural) {
              (false, false, false, false, false, false, false) => vec!["um"],
              (true,  false, false, false, false, false, false) => vec!["um"],
              (false, true,  false, false, false, false, false) => vec!["ium"],
              (true,  true,  false, false, false, false, false) => vec!["ium", "um"],
              (true,  true,  true,  false, false, false, false) => vec!["ium"],
              (false, true,  false, true,  false, false, false) => vec!["ium"],
              (false, true,  false, false, true,  false, false) => vec!["ium"],
              (false, false, false, false, false, true,  true ) => vec!["ium"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::DatPl, from_stem_or_lemma(vec!["ibus"]));
          map.insert(Case::AccPl, from_stem_or_lemma(match
            (  neuter,i,     pure,  ignis, navis, loc,   plural) {
              (false, false, false, false, false, false, false) => vec!["ēs"],
              (true,  false, false, false, false, false, false) => vec!["a"],
              (false, true,  false, false, false, false, false) => vec!["ēs"],
              (true,  true,  false, false, false, false, false) => vec!["a"],
              (true,  true,  true,  false, false, false, false) => vec!["ia"],
              (false, true,  false, true,  false, false, false) => vec!["ēs", "īs"],
              (false, true,  false, false, true,  false, false) => vec!["ēs", "īs"],
              (false, false, false, false, false, true,  true ) => vec!["es"],
              _ => panic!("bad subtypes"),
          }));
          map.insert(Case::AblPl, from_stem_or_lemma(vec!["ibus"]));
          map.insert(Case::VocPl, from_stem_or_lemma(match
            (  neuter,i,     pure,  ignis, navis, loc,   plural) {
              (false, false, false, false, false, false, false) => vec!["ēs"],
              (true,  false, false, false, false, false, false) => vec!["a"],
              (false, true,  false, false, false, false, false) => vec!["ēs"],
              (true,  true,  false, false, false, false, false) => vec!["a"],
              (true,  true,  true,  false, false, false, false) => vec!["ia"],
              (false, true,  false, true,  false, false, false) => vec!["ēs"],
              (false, true,  false, false, true,  false, false) => vec!["ēs"],
              (false, false, false, false, false, true,  true ) => vec!["es"],
              _ => panic!("bad subtypes"),
          }));
          if *loc {
            map.insert(Case::LocPl, from_stem_or_lemma(vec!["ibus"]));
          }
        }
        if *plural {
          map.insert(Case::NomPl, from_stem_or_lemma(vec![]));
        } else {
          map.insert(Case::NomSg, from_stem_or_lemma(vec![]));
        }
      },
      Self::D4 { neuter, greek, plural, single } => {
        match (neuter, greek) {
          (false, false) => {
            if !*plural {
              map.insert(Case::GenSg, from_stem_or_lemma(vec!["ūs"]));
              map.insert(Case::DatSg, from_stem_or_lemma(vec!["uī"]));
              map.insert(Case::AccSg, from_stem_or_lemma(vec!["um"]));
              map.insert(Case::AblSg, from_stem_or_lemma(vec!["ū"]));
              map.insert(Case::VocSg, from_stem_or_lemma(vec!["us"]));
            }
            if !*single {
              map.insert(Case::NomPl, from_stem_or_lemma(vec!["ūs"]));
              map.insert(Case::GenPl, from_stem_or_lemma(vec!["uum"]));
              map.insert(Case::DatPl, from_stem_or_lemma(vec!["ibus"]));
              map.insert(Case::AccPl, from_stem_or_lemma(vec!["ūs"]));
              map.insert(Case::AblPl, from_stem_or_lemma(vec!["ibus"]));
              map.insert(Case::VocPl, from_stem_or_lemma(vec!["ūs"]));
            }
          },
          (true, false) => {
            if !*plural {
              map.insert(Case::GenSg, from_stem_or_lemma(vec!["ūs", "ū"]));
              map.insert(Case::DatSg, from_stem_or_lemma(vec!["ūī", "ū"]));
              map.insert(Case::AccSg, from_stem_or_lemma(vec!["ū"]));
              map.insert(Case::AblSg, from_stem_or_lemma(vec!["ū"]));
              map.insert(Case::VocSg, from_stem_or_lemma(vec!["ū"]));
            }
            if !*single {
              map.insert(Case::NomPl, from_stem_or_lemma(vec!["ua"]));
              map.insert(Case::GenPl, from_stem_or_lemma(vec!["uum"]));
              map.insert(Case::DatPl, from_stem_or_lemma(vec!["ibus"]));
              map.insert(Case::AccPl, from_stem_or_lemma(vec!["ua"]));
              map.insert(Case::AblPl, from_stem_or_lemma(vec!["ibus"]));
              map.insert(Case::VocPl, from_stem_or_lemma(vec!["ua"]));
            }
          },
          (false, true) => {
            map.insert(Case::GenSg, from_stem_or_lemma(vec!["ūs"]));
            map.insert(Case::DatSg, from_stem_or_lemma(vec!["ō"]));
            map.insert(Case::AccSg, from_stem_or_lemma(vec!["ō"]));
            map.insert(Case::AblSg, from_stem_or_lemma(vec!["ō"]));
            map.insert(Case::VocSg, from_stem_or_lemma(vec!["ō"]));
          },
          _ => panic!("bad subtypes"),
        }
        if *plural {
          map.insert(Case::NomPl, from_stem_or_lemma(vec![]));
        } else {
          map.insert(Case::NomSg, from_stem_or_lemma(vec![]));
        }
      },
      Self::D5 { ies, plural, single } => {
        if !*plural {
          map.insert(Case::GenSg, from_stem_or_lemma(match
            ies {
              true  => vec!["ēī"],
              false => vec!["eī"],
          }));
          map.insert(Case::DatSg, from_stem_or_lemma(match
            ies {
              true  => vec!["ēī"],
              false => vec!["eī"],
          }));
          map.insert(Case::AccSg, from_stem_or_lemma(vec!["em"]));
          map.insert(Case::AblSg, from_stem_or_lemma(vec!["ē"]));
          map.insert(Case::VocSg, from_stem_or_lemma(vec!["ēs"]));
          map.insert(Case::LocSg, from_stem_or_lemma(vec!["ē"]));
        }
        if !*single {
          map.insert(Case::NomPl, from_stem_or_lemma(vec!["ēs"]));
          map.insert(Case::GenPl, from_stem_or_lemma(vec!["ērum"]));
          map.insert(Case::DatPl, from_stem_or_lemma(vec!["ēbus"]));
          map.insert(Case::AccPl, from_stem_or_lemma(vec!["ēs"]));
          map.insert(Case::AblPl, from_stem_or_lemma(vec!["ēbus"]));
          map.insert(Case::VocPl, from_stem_or_lemma(vec!["ēs"]));
          map.insert(Case::LocPl, from_stem_or_lemma(vec!["ēbus"]));
        }
        if *plural {
          map.insert(Case::NomPl, from_stem_or_lemma(vec![]));
        } else {
          map.insert(Case::NomSg, from_stem_or_lemma(vec![]));
        }
      },
      _ => {
        map.insert(Case::NomSg, from_stem_or_lemma(vec![]));
      },
    }

    map
  }
}

fn latina(word: &str) -> TemplateText {
  let noun = if let Ok(one_lemma) = env_var("ENV_1") {
    Noun::new(word, one_lemma)
  } else {
    let mut nouns = Vec::new();
    let mut i = 0;
    while let Ok(lemma) = env_var(&format!("ENV_1_{}", i)) {
      i += 1;
      nouns.push(Noun::new(word, lemma));
    }
    let mut nouns = nouns.into_iter();
    let mut noun = nouns.next().unwrap();
    for variant in nouns {
      noun.merge(variant);
    }
    noun
  };

  let mut subwords = Vec::new();
  if let Ok(word) = env_var("ENV_f") { subwords.push(word); }
  if let Ok(word) = env_var("ENV_m") { subwords.push(word); }

  TemplateText {
    mutation: Some(noun.table),
    lemma: Some(noun.lemmas[0].clone()),
    tags: noun.tags.into_iter().map(|v| v.to_string()).collect(),
    notes: noun.footnote,
    conjugation: None,
    pronunciation: None,
    meanings: None,
    examples: None,
    subwords,
  }
}

// lemma stem declension
fn parse_lemma(src: &str) -> (String, Option<String>, String) {
  let mut lemmas = vec![String::new()];
  for c in src.chars() {
    match c {
      '<' | '/' => {
        lemmas.push(String::new());
      },
      '>' => { break; },
      x => lemmas.last_mut().unwrap().push(x),
    }
  }
  let decl = lemmas.pop().unwrap();
  let stem = lemmas.pop().unwrap();
  if let Some(lemma) = lemmas.pop() {
    (lemma, Some(stem), decl)
  } else {
    (stem, None, decl)
  }
}

// declension subtypes
fn parse_declension(src: &str) -> (String, Vec<String>) {
  let mut iter = src.split('.');
  (iter.next().unwrap().to_string(), iter.map(|v| v.to_owned()).collect())
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Case {
  NomSg,
  GenSg,
  DatSg,
  AccSg,
  AblSg,
  VocSg,
  LocSg,
  NomPl,
  GenPl,
  DatPl,
  AccPl,
  AblPl,
  VocPl,
  LocPl,
}

impl ToString for Case {
  fn to_string(&self) -> String {
    match self {
      Self::NomSg => "nom.sg.".to_owned(),
      Self::GenSg => "gen.sg.".to_owned(),
      Self::DatSg => "dat.sg.".to_owned(),
      Self::AccSg => "acc.sg.".to_owned(),
      Self::AblSg => "abl.sg.".to_owned(),
      Self::VocSg => "voc.sg.".to_owned(),
      Self::LocSg => "loc.sg.".to_owned(),
      Self::NomPl => "nom.pl.".to_owned(),
      Self::GenPl => "gen.pl.".to_owned(),
      Self::DatPl => "dat.pl.".to_owned(),
      Self::AccPl => "acc.pl.".to_owned(),
      Self::AblPl => "abl.pl.".to_owned(),
      Self::VocPl => "voc.pl.".to_owned(),
      Self::LocPl => "loc.pl.".to_owned(),
    }
  }
}
