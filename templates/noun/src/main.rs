use template::*;
use serde_json::to_writer;
use std::env::var as env_var;
use std::collections::{HashSet, HashMap};

fn main() {
  let word = env_var("ENV_MAINWORD").unwrap();
  let lang = env_var("ENV_0").unwrap();

  let data = match lang.as_str() {
    "la-noun" | "la-proper noun" => latina(&word, Request::Lemma),
    "la-ndecl" => latina(&word, Request::Table),
    u => panic!("unsupported: {}", u),
  };

  to_writer(std::io::stdout(), &data).unwrap();
}

#[derive(Debug, Clone, Copy)]
enum Num {
  Single,
  Plural,
  Both,
}

#[derive(Debug, Clone, Copy)]
enum Gender {
  Male,
  Female,
  Neuter,
  Common,
}

#[derive(Debug, Default, Clone)]
struct Genders {
  male_single: bool,
  female_single: bool,
  neuter_single: bool,
  male_plural: bool,
  female_plural: bool,
  neuter_plural: bool,
}

impl Genders {
  fn set(&mut self, gender: (Gender, Num)) {
    match gender {
      (gender, Num::Both) => {
        self.set((gender, Num::Single));
        self.set((gender, Num::Plural));
      },
      (Gender::Common, num) => {
        self.set((Gender::Male, num));
        self.set((Gender::Female, num));
      },
      (Gender::Male, Num::Single) => self.male_single = true,
      (Gender::Female, Num::Single) => self.female_single = true,
      (Gender::Neuter, Num::Single) => self.neuter_single = true,
      (Gender::Male, Num::Plural) => self.male_plural = true,
      (Gender::Female, Num::Plural) => self.female_plural = true,
      (Gender::Neuter, Num::Plural) => self.neuter_plural = true,
    }
  }

  fn extend(&mut self, num: Num) {
    match num {
      Num::Single => {
        self.male_plural = false;
        self.female_plural = false;
        self.neuter_plural = false;
      },
      Num::Plural => {
        self.male_single = false;
        self.female_single = false;
        self.neuter_single = false;
      },
      Num::Both => {
        let male = self.male_plural || self.male_single;
        let female = self.female_plural || self.female_single;
        let neuter = self.neuter_plural || self.neuter_single;
        self.male_plural = male;
        self.female_plural = female;
        self.neuter_plural = neuter;
        self.male_single = male;
        self.female_single = female;
        self.neuter_single = neuter;
      },
    }
  }
}

#[derive(Debug, Clone)]
struct Noun {
  word: String,
  stem: String,
  lemmas: Vec<String>,
  footnote: Option<String>,
  title: Option<String>,
  genders: Genders,
  declension: String,
  table: HashMap<String, String>,
  tags: HashSet<String>,
}

impl Noun {
  fn new(word: &str) -> Self {
    let lemma = env_var("ENV_1").unwrap();
    let (lemma, stem, decl) = parse_lemma(&lemma);

    let mut lemmas = vec![
      env_var("ENV_lemma").unwrap_or_else(|_| lemma.clone())
    ];
    let mut i = 1;
    while let Ok(lemma) = env_var(&format!("ENV_lemma{}", i)) {
      i += 1;
      lemmas.push(lemma);
    }

    let (decl, subdecl) = parse_declension(&decl);
    let mut declension = match decl.as_str() {
      "1" => Declension::d1(),
      "2" => Declension::d2(),
      "3" => Declension::d3(),
      "4" => Declension::d4(),
      "5" => Declension::d5(),
      "irreg" => Declension::Irreg {},
      "indecl" | "0" => Declension::Indecl {},
      u => panic!("unknown declension: {}", u),
    };

    let lemma_end = |lemma: &str, end: &str, stem_end: &str| -> Option<String> {
      lemma.strip_suffix(end).map(|s| s.to_owned() + stem_end)
    };

    let stem = stem.unwrap_or_else(|| {
      match &declension {
        Declension::D1 { .. } => {
          lemma_end(&lemma, "a", "")
            .or_else(|| lemma_end(&lemma, "ē", ""))
            .or_else(|| lemma_end(&lemma, "ās", ""))
            .or_else(|| lemma_end(&lemma, "ēs", ""))
            .or_else(|| lemma_end(&lemma, "ām", ""))
            .or_else(|| lemma_end(&lemma, "ae", ""))
            .expect("unknown 1 declension ending")
        },
        Declension::D2 { .. } => {
          lemma_end(&lemma, "r", "r")
            .or_else(|| lemma_end(&lemma, "us", ""))
            .or_else(|| lemma_end(&lemma, "um", ""))
            .or_else(|| lemma_end(&lemma, "os", ""))
            .or_else(|| lemma_end(&lemma, "on", ""))
            .or_else(|| lemma_end(&lemma, "ī", ""))
            .or_else(|| lemma_end(&lemma, "a", ""))
            .expect("unknown 2 declension ending")
        },
        Declension::D3 { .. } => {
          lemma_end(&lemma, "e", "")
            .or_else(|| lemma_end(&lemma, "ō", "ōn"))
            .or_else(|| lemma_end(&lemma, "s", "t"))
            .or_else(|| lemma_end(&lemma, "x", "c"))
            .or_else(|| lemma_end(&lemma, "al", "āl"))
            .or_else(|| lemma_end(&lemma, "ar", "ār"))
            .or_else(|| lemma_end(&lemma, "er", "r"))
            .or_else(|| lemma_end(&lemma, "or", "ōr"))
            .or_else(|| lemma_end(&lemma, "gō", "gin"))
            .or_else(|| lemma_end(&lemma, "ps", "p"))
            .or_else(|| lemma_end(&lemma, "bs", "b"))
            .or_else(|| lemma_end(&lemma, "is", ""))
            .or_else(|| lemma_end(&lemma, "ēs", ""))
            .or_else(|| lemma_end(&lemma, "us", "or"))
            .or_else(|| lemma_end(&lemma, "ex", "ic"))
            .or_else(|| lemma_end(&lemma, "ma", "mat"))
            .or_else(|| lemma_end(&lemma, "āns", "ant"))
            .or_else(|| lemma_end(&lemma, "ēns", "ent"))
            .or_else(|| lemma_end(&lemma, "ōns", "ont"))
            .or_else(|| lemma_end(&lemma, "men", "min"))
            .or_else(|| lemma_end(&lemma, "ceps", "cipit"))
            .or_else(|| lemma_end(&lemma, "tūdō", "tūdin"))
            .unwrap_or_else(|| lemma.clone())
        },
        Declension::D4 { .. } => {
          lemma_end(&lemma, "us", "")
            .or_else(|| lemma_end(&lemma, "ūs", ""))
            .or_else(|| lemma_end(&lemma, "ū", ""))
            .or_else(|| lemma_end(&lemma, "ua", ""))
            .or_else(|| lemma_end(&lemma, "ō", ""))
            .expect("unknown 4 declension ending")
        },
        Declension::D5 { .. } => {
          lemma_end(&lemma, "ēs", "")
            .expect("unknown 5 declension ending")
        },
        Declension::Indecl { .. } => { lemma.clone() },
        Declension::Irreg { .. } => {
          panic!("irregular without stem")
        },
      }
    });

    if env_var("ENV_indecl").is_ok() { declension = Declension::Indecl {} };


    let mut gender = GenderRule::default();
    let mut tantum = TantumRule::default();
    let mut greek = Rule::default();
    let mut loc = Rule::default();
    let mut abus_type1 = Rule::default();
    let mut am_type1 = Rule::default();
    let mut ma_type1 = Rule::default();
    let mut me_type1 = Rule::default();
    let mut er_type2 = Rule::default();
    let mut vos_type2 = Rule::default();
    let mut vom_type2 = Rule::default();
    let mut us_type2 = Rule::default();
    let mut ius_type2 = Rule::default();
    let mut ium_type2 = Rule::default();
    let mut voci_type2 = Rule::default();
    let mut pure_type3 = Rule::default();
    let mut i_type3 = Rule::default();
    let mut ignis_type3 = Rule::default();
    let mut navis_type3 = Rule::default();
    let mut acc_im_type3 = Rule::default();
    let mut acc_im_in_type3 = Rule::default();
    let mut acc_im_in_em_type3 = Rule::default();
    let mut acc_im_em_type3 = Rule::default();
    let mut acc_im_occ_em_type3 = Rule::default();
    let mut acc_em_im_type3 = Rule::default();
    let mut abl_i_type3 = Rule::default();
    let mut abl_i_e_type3 = Rule::default();
    let mut abl_e_i_type3 = Rule::default();
    let mut abl_e_occ_i_type3 = Rule::default();
    let mut callisto_type4 = Rule::default();
    let mut ies_type5 = Rule::default();

    for sub in subdecl {
      match sub.as_str() {
        "sg" => tantum.set_single(true),
        "-sg" => tantum.set_single(false),
        "pl" => tantum.set_plural(true),
        "-pl" => tantum.set_plural(false),
        "both" => tantum.set_both(true),
        "-both" => tantum.set_both(false),
        "N" => gender.set_neuter(true),
        "-N" => gender.set_neuter(false),
        "M" => gender.set_male(true),
        "-M" => gender.set_male(false),
        "F" => gender.set_female(true),
        "-F" => gender.set_female(false),
        "loc" => loc = Rule::Set,
        "-loc" => loc = Rule::Forbidden,
        "I" => i_type3 = Rule::Set,
        "-I" => i_type3 = Rule::Forbidden,
        "pure" => pure_type3 = Rule::Set,
        "-pure" => pure_type3 = Rule::Forbidden,
        "navis" => navis_type3 = Rule::Set,
        "-navis" => navis_type3 = Rule::Forbidden,
        "ignis" => ignis_type3 = Rule::Set,
        "-ignis" => ignis_type3 = Rule::Forbidden,
        "Greek" => greek = Rule::Set,
        "-Greek" => greek = Rule::Forbidden,
        "abus" => abus_type1 = Rule::Set,
        "-abus" => abus_type1 = Rule::Forbidden,
        "am" => am_type1 = Rule::Set,
        "-am" => am_type1 = Rule::Forbidden,
        "ma" => ma_type1 = Rule::Set,
        "-ma" => ma_type1 = Rule::Forbidden,
        "me" => me_type1 = Rule::Set,
        "-me" => me_type1 = Rule::Forbidden,
        "er" => er_type2 = Rule::Set,
        "-er" => er_type2 = Rule::Forbidden,
        "vos" => vos_type2 = Rule::Set,
        "-vos" => vos_type2 = Rule::Forbidden,
        "vom" => vom_type2 = Rule::Set,
        "-vom" => vom_type2 = Rule::Forbidden,
        "us" => us_type2 = Rule::Set,
        "-us" => us_type2 = Rule::Forbidden,
        "ius" => ius_type2 = Rule::Set,
        "-ius" => ius_type2 = Rule::Forbidden,
        "ium" => ium_type2 = Rule::Set,
        "-ium" => ium_type2 = Rule::Forbidden,
        "voci" => voci_type2 = Rule::Set,
        "-voci" => voci_type2 = Rule::Forbidden,
        "ies" => ies_type5 = Rule::Set,
        "-ies" => ies_type5 = Rule::Forbidden,
        "Callisto" => callysto_type4 = Rule::Set,
        // i-stem manual types
        "acc-im" => { acc_im_type3 = Rule::Set; i_type3 = Rule::Set; },
        "acc-im-in" => { acc_im_in_type3 = Rule::Set; i_type3 = Rule::Set; },
        "acc-im-in-em" => { acc_im_in_em_type3 = Rule::Set; i_type3 = Rule::Set; },
        "acc-im-em" => { acc_im_em_type3 = Rule::Set; i_type3 = Rule::Set; },
        "acc-im-occ-em" => { acc_im_occ_em_type3 = Rule::Set; i_type3 = Rule::Set; },
        "acc-em-im" => { acc_em_im_type3 = Rule::Set; i_type3 = Rule::Set; },
        "abl-i" => { abl_i_type3 = Rule::Set; i_type3 = Rule::Set; },
        "abl-i-e" => { abl_i_e_type3 = Rule::Set; i_type3 = Rule::Set; },
        "abl-e-i" => { abl_e_i_type3 = Rule::Set; i_type3 = Rule::Set; },
        "abl-e-occ-i" => { abl_e_occ_i_type3 = Rule::Set; i_type3 = Rule::Set; },
        u => panic!("unknown subdeclension: {}", u),
      }
    }

    let autorule = |subs: &mut [&mut Rule]| -> bool {
      for sub in subs.iter() {
        if **sub == Rule::Forbidden { return false; }
      }
      for sub in subs {
        if **sub == Rule::Auto { **sub = Rule::Set; }
      }
      true
    };

    match &declension {
      Declension::D1 {..} => {
        if lemma.ends_with("ām") {
          autorule(&mut [&mut gender.female, &mut am_type1]);
        } else if lemma.ends_with("ās") {
          autorule(&mut [&mut gender.male, &mut greek, &mut ma_type1]);
        } else if lemma.ends_with("ēs") {
          autorule(&mut [&mut gender.male, &mut greek, &mut me_type1]);
        } else if lemma.ends_with("ē") {
          autorule(&mut [&mut gender.female, &mut greek]);
        } else if lemma.ends_with("ae") {
          autorule(&mut [&mut gender.female, &mut tantum.plural]);
        } else if lemma.ends_with("a") {
          autorule(&mut [&mut gender.female]);
        }
      },
      Declension::D2 {..} => {
        if lemma.ends_with("r") {
          autorule(&mut [&mut gender.male, &mut er_type2]);
        } else if lemma.ends_with("vos") {
          autorule(&mut [&mut gender.male, &mut vos_type2]);
        } else if lemma.ends_with("vom") {
          autorule(&mut [&mut gender.neuter, &mut vom_type2]);
        } else if lemma.ends_with("os") {
          autorule(&mut [&mut gender.male, &mut greek]);
          if gender.male == Rule::Forbidden {
            autorule(&mut [&mut gender.neuter, &mut greek, &mut us_type2]);
          }
        } else if lemma.ends_with("on") {
          autorule(&mut [&mut gender.neuter, &mut greek]);
        } else if lemma.ends_with("us") {
          if lemma.ends_with("ius") && 
            !if lemma.chars().next().unwrap().is_uppercase() {
              autorule(&mut [&mut gender.male, &mut ius_type2, &mut voci_type2, &mut tantum.single])
            } else {
              autorule(&mut [&mut gender.male, &mut ius_type2])
            }
          || !lemma.ends_with("ius") {
            if !autorule(&mut [&mut gender.male]) {
              if !autorule(&mut [&mut gender.neuter, &mut us_type2]) {
                autorule(&mut [&mut gender.neuter, &mut us_type2, &mut tantum.plural]);
              }
            }
          }
        } else if lemma.ends_with("um") {
          if lemma.ends_with("ium") &&
            !autorule(&mut [&mut gender.neuter, &mut ium_type2])
            || !lemma.ends_with("ium") {
              autorule(&mut [&mut gender.neuter]);
            }
        } else if lemma.ends_with("ī") {
          if lemma.ends_with("iī") &&
            !autorule(&mut [&mut gender.male, &mut ius_type2, &mut tantum.plural])
            || !lemma.ends_with("iī") {
              autorule(&mut [&mut gender.male, &mut tantum.plural]);
            }
        } else if lemma.ends_with("a") {
          if lemma.ends_with("ia") &&
            !autorule(&mut [&mut gender.neuter, &mut ium_type2, &mut tantum.plural])
            || !lemma.ends_with("ia") {
              autorule(&mut [&mut gender.neuter, &mut tantum.plural]);
            }
        }
      },
      Declension::D3 {..} => {
        if lemma.ends_with("tūdo") && stem.ends_with("tūdin") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("tās") && stem.ends_with("tāt") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("tūs") && stem.ends_with("tūt") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("tiō") && stem.ends_with("tiōn") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("siō") && stem.ends_with("siōn") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("xiō") && stem.ends_with("xiōn") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("gō") && stem.ends_with("gin") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("or") && stem.ends_with("ōr") {
          autorule(&mut [&mut gender.male]);
        } else if lemma.ends_with("trīx") && stem.ends_with("trīc") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("is") && lemma.starts_with(&stem) && gender.neuter != Rule::Set {
          autorule(&mut [&mut i_type3]);
        } else if lemma.ends_with("ēs") && lemma.starts_with(&stem) && gender.neuter != Rule::Set && lemma.chars().next().unwrap().is_lowercase() {
          autorule(&mut [&mut i_type3]);
        } else if lemma.ends_with("us") && stem.ends_with("or") {
          autorule(&mut [&mut gender.neuter]);
        } else if lemma.ends_with("us") && stem.ends_with("er") {
          autorule(&mut [&mut gender.neuter]);
        } else if lemma.ends_with("ma") && stem.ends_with("mat") {
          autorule(&mut [&mut gender.neuter]);
        } else if lemma.ends_with("men") && stem.ends_with("min") {
          autorule(&mut [&mut gender.neuter]);
        } else if lemma.ends_with("e") && lemma.starts_with(&stem) {
          if lemma.chars().next().unwrap().is_uppercase() {
            autorule(&mut [&mut gender.neuter]);
          } else {
            autorule(&mut [&mut gender.neuter, &mut i_type3, &mut pure_type3]);
          }
        } else if lemma.ends_with("al") && stem.ends_with("āl") {
          autorule(&mut [&mut gender.neuter, &mut i_type3, &mut pure_type3]);
        } else if lemma.ends_with("ar") && stem.ends_with("ār") {
          autorule(&mut [&mut gender.neuter, &mut i_type3, &mut pure_type3]);
        }
      },
      Declension::D4 { .. } => {
        if lemma.ends_with("us") {
          autorule(&mut [&mut gender.female]);
        } else if lemma.ends_with("ū") {
          autorule(&mut [&mut gender.neuter]);
        } else if lemma.ends_with("ō") {
          autorule(&mut [&mut gender.female, &mut greek]);
        }
      },
      Declension::D5 { .. } => {
        autorule(&mut [&mut gender.female]);
        if lemma.ends_with("iēs") {
          autorule(&mut [&mut ies_type5]);
        }
      },
      _ => {},
    }

    let num = env_var("ENV_num").map(|v| {
      match v.as_str() {
        "sg" => Num::Single,
        "pl" => Num::Plural,
        "both" => Num::Both,
        u => panic!("unsupported num: {}", u),
      }
    });

    let footnote = env_var("ENV_footnote").ok();
    let title = env_var("ENV_title").ok();

    let mut genders = Genders::default();
    let scan_gender = |g: String| {
      match g.as_str() {
        "m" => (Gender::Male, Num::Both),
        "f" => (Gender::Female, Num::Both),
        "n" => (Gender::Neuter, Num::Both),
        "c" => (Gender::Common, Num::Both),
        "m-s" => (Gender::Male, Num::Single),
        "f-s" => (Gender::Female, Num::Single),
        "n-s" => (Gender::Neuter, Num::Single),
        "c-s" => (Gender::Common, Num::Single),
        "m-p" => (Gender::Male, Num::Plural),
        "f-p" => (Gender::Female, Num::Plural),
        "n-p" => (Gender::Neuter, Num::Plural),
        "c-p" => (Gender::Common, Num::Plural),
        u => panic!("unknown gender: {}", u),
      }
    };

    if env_var("ENV_g").map(scan_gender).map(|v| genders.set(v)).is_ok() {
      let mut i = 1;
      while let Ok(gender) = env_var(&format!("ENV_g{}", i)) {
        i += 1;
        genders.set(scan_gender(gender));
      }
    } else {
      let num = if tantum.single == Rule::Set { Num::Single }
      else if tantum.plural == Rule::Set { Num::Plural }
      else { Num::Both };
      if gender.male == Rule::Set { genders.set((Gender::Male, num)); }
      if gender.female == Rule::Set { genders.set((Gender::Female, num)); }
      if gender.neuter == Rule::Set { genders.set((Gender::Neuter, num)); }
    }

    if let Ok(num) = num { genders.extend(num); }

    let declension = match declension {
      Declension::D1 {..} => Declension::D1 {
        abus: abus_type1.into(),
        am: am_type1.into(),
        ma: ma_type1.into(),
        me: me_type1.into(),
        greek: greek.into(),
        loc: loc.into(),
      },
      Declension::D2 {..} => Declension::D2 {
        er: er_type2.into(),
        ius: ius_type2.into(),
        voci: voci_type2.into(),
        ium: ium_type2.into(),
        us: us_type2.into(),
        greek: greek.into(),
        loc: loc.into(),
        neuter: gender.neuter.into(),
      },
      Declension::D3 {..} => Declension::D3 {
        i: i_type3.into(),
        pure: pure_type3.into(),
        ignis: ignis_type3.into(),
        navis: navis_type3.into(),
        loc: loc.into(),
        plural: tantum.plural.into(),
        neuter: gender.neuter.into(),
        acc_im: acc_im_type3.into(),
        acc_im_in: acc_im_in_type3.into(),
        acc_im_in_em: acc_im_in_em_type3.into(),
        acc_im_em: acc_im_em_type3.into(),
        acc_im_occ_em: acc_im_occ_em_type3.into(),
        acc_em_im: acc_em_im_type3.into(),
        abl_i: abl_i_type3.into(),
        abl_i_e: abl_i_e_type3.into(),
        abl_e_i: abl_e_i_type3.into(),
        abl_e_occ_i: abl_e_occ_i_type3.into(),
      },
      Declension::D4 {..} => Declension::D4 {
        neuter: gender.neuter.into(),
        greek: greek.into(),
      },
      Declension::D5 {..} => Declension::D5 {
        ies: ies_type5.into(),
      },
      n => n,
    };

    let (declension_id, mut tags) = declension.encode();
    let mut table: HashMap<String, String> = declension.table(&lemma, &stem).into_iter().map(|(id, val)| (id.to_string(), val)).collect();
    if gender.male.into() { tags.insert("male".to_owned()); }
    if gender.female.into() { tags.insert("female".to_owned()); }
    if gender.neuter.into() { tags.insert("neuter".to_owned()); }
    if tantum.single.into() { tags.insert("single".to_owned()); }
    if tantum.plural.into() { tags.insert("plural".to_owned()); }
    if tantum.both.into() { tags.insert("plural".to_owned()); tags.insert("single".to_owned()); }
    if greek.into() { tags.insert("greek".to_owned()); }
    if loc.into() { tags.insert("loc".to_owned()); }

    if let Ok(v) = env_var("ENV_nom_sg") { table.insert(Case::NomSg.to_string(), v); }
    if let Ok(v) = env_var("ENV_gen_sg") { table.insert(Case::GenSg.to_string(), v); }
    if let Ok(v) = env_var("ENV_dat_sg") { table.insert(Case::DatSg.to_string(), v); }
    if let Ok(v) = env_var("ENV_acc_sg") { table.insert(Case::AccSg.to_string(), v); }
    if let Ok(v) = env_var("ENV_abl_sg") { table.insert(Case::AblSg.to_string(), v); }
    if let Ok(v) = env_var("ENV_voc_sg") { table.insert(Case::VocSg.to_string(), v); }
    if let Ok(v) = env_var("ENV_loc_sg") { table.insert(Case::LocSg.to_string(), v); }
    if let Ok(v) = env_var("ENV_nom_pl") { table.insert(Case::NomPl.to_string(), v); }
    if let Ok(v) = env_var("ENV_gen_pl") { table.insert(Case::GenPl.to_string(), v); }
    if let Ok(v) = env_var("ENV_dat_pl") { table.insert(Case::DatPl.to_string(), v); }
    if let Ok(v) = env_var("ENV_acc_pl") { table.insert(Case::AccPl.to_string(), v); }
    if let Ok(v) = env_var("ENV_abl_pl") { table.insert(Case::AblPl.to_string(), v); }
    if let Ok(v) = env_var("ENV_voc_pl") { table.insert(Case::VocPl.to_string(), v); }
    if let Ok(v) = env_var("ENV_loc_pl") { table.insert(Case::LocPl.to_string(), v); }

    Self {
      word: word.to_owned(),
      lemmas,
      footnote,
      title,
      genders,
      declension: declension_id,
      stem,
      tags,
      table,
    }
  }
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

#[derive(Default)]
struct GenderRule {
  male: Rule,
  female: Rule,
  neuter: Rule,
}

impl GenderRule {
  fn set_male(&mut self, sign: bool) {
    if sign {
      self.male = Rule::Set;
      self.female = Rule::Allow;
      self.neuter = Rule::Forbidden;
    } else {
      self.male = Rule::Forbidden;
      self.female = Rule::Allow;
      self.neuter = Rule::Allow;
    }
  }

  fn set_female(&mut self, sign: bool) {
    if sign {
      self.male = Rule::Allow;
      self.female = Rule::Set;
      self.neuter = Rule::Forbidden;
    } else {
      self.male = Rule::Allow;
      self.female = Rule::Forbidden;
      self.neuter = Rule::Allow;
    }
  }

  fn set_neuter(&mut self, sign: bool) {
    if sign {
      self.male = Rule::Forbidden;
      self.female = Rule::Forbidden;
      self.neuter = Rule::Set;
    } else {
      self.male = Rule::Allow;
      self.female = Rule::Allow;
      self.neuter = Rule::Forbidden;
    }
  }
}

#[derive(Default)]
struct TantumRule {
  single: Rule,
  both: Rule,
  plural: Rule,
}

impl TantumRule {
  fn set_single(&mut self, sign: bool) {
    if sign {
      self.single = Rule::Set;
      self.both = Rule::Allow;
      self.plural = Rule::Forbidden;
    } else {
      self.single = Rule::Forbidden;
      self.both = Rule::Allow;
      self.plural = Rule::Allow;
    }
  }

  fn set_both(&mut self, sign: bool) {
    if sign {
      self.single = Rule::Allow;
      self.both = Rule::Set;
      self.plural = Rule::Allow;
    } else {
      self.single = Rule::Allow;
      self.both = Rule::Forbidden;
      self.plural = Rule::Allow;
    }
  }

  fn set_plural(&mut self, sign: bool) {
    if sign {
      self.single = Rule::Forbidden;
      self.both = Rule::Allow;
      self.plural = Rule::Set;
    } else {
      self.single = Rule::Allow;
      self.both = Rule::Allow;
      self.plural = Rule::Forbidden;
    }
  }
}

#[derive(Debug, Clone)]
enum Declension {
  D1 {
    abus: bool,
    greek: bool,
    ma: bool,
    me: bool,
    loc: bool,
    am: bool,
  },
  D2 {
    neuter: bool,
    er: bool,
    greek: bool,
    ius: bool,
    voci: bool,
    ium: bool,
    loc: bool,
    us: bool,
  },
  D3 {
    neuter: bool,
    i: bool,
    pure: bool,
    ignis: bool,
    navis: bool,
    loc: bool,
    plural: bool,
    acc_im: bool,
    acc_im_in: bool,
    acc_im_in_em: bool,
    acc_im_em: bool,
    acc_im_occ_em: bool,
    acc_em_im: bool,
    abl_i: bool,
    abl_i_e: bool,
    abl_e_i: bool,
    abl_e_occ_i: bool,
  },
  D4 {
    neuter: bool,
    greek: bool
  },
  D5 {
    ies: bool,
  },
  Irreg {},
  Indecl {},
}

impl Declension {
  fn encode(&self) -> (String, HashSet<String>) {
    let mut tags = HashSet::new();
    
    (match self {
      Self::D1 { abus, greek, ma, me, loc, am } => {
        tags.insert("decl_1".to_owned());
        if *abus { tags.insert("d1_abus".to_owned()); }
        if *greek { tags.insert("greek".to_owned()); }
        if *ma { tags.insert("d1_ma".to_owned()); }
        if *me { tags.insert("d1_me".to_owned()); }
        if *loc { tags.insert("loc".to_owned()); }
        if *am { tags.insert("d1_am".to_owned()); }
        "1"
      },
      Self::D2 { neuter, greek, ius, er, voci, ium, loc, us } => {
        tags.insert("decl_2".to_owned());
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
      Self::D3 { neuter, i, pure, ignis, navis, loc, plural, acc_im, acc_im_in, acc_im_in_em, acc_im_em, acc_im_occ_em, acc_em_im, abl_i, abl_i_e, abl_e_i, abl_e_occ_i } => {
        tags.insert("decl_3".to_owned());
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
      Self::D4 { neuter, greek } => {
        tags.insert("decl_4".to_owned());
        if *neuter { tags.insert("neuter".to_owned()); }
        if *greek { tags.insert("greek".to_owned()); }
        "4"
      },
      Self::D5 { ies } => {
        tags.insert("decl_5".to_owned());
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
    }
  }

  fn d4() -> Self {
    Self::D4 {
      neuter: false,
      greek: false,
    }
  }

  fn d5() -> Self {
    Self::D5 {
      ies: false,
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

    map.insert(Case::NomSg, from_stem_or_lemma(vec![]));
    match self {
      Self::D1 { abus, greek, ma, me, loc, am } => {
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
          map.insert(Case::LocSg, from_stem_or_lemma(vec!["ae"]));
          map.insert(Case::LocPl, from_stem_or_lemma(vec!["īs"]));
        }
      },
      Self::D2 { neuter, er, greek, ius, voci, ium, loc, us } => {
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
        if !(*neuter && *us) {
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
        }
        if *loc {
          map.insert(Case::LocSg, from_stem_or_lemma(vec!["ī"]));
          map.insert(Case::LocPl, from_stem_or_lemma(vec!["īs"]));
        }
      },
      Self::D3 { neuter, i, pure, ignis, navis, loc, plural, acc_im, acc_im_in, acc_im_in_em, acc_im_em, acc_im_occ_em, acc_em_im, abl_i, abl_i_e, abl_e_i, abl_e_occ_i } => {
        if !(*loc && *plural) {
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
        if !(*loc && !*plural) {
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
      },
      Self::D4 { neuter, greek } => {
        match (neuter, greek) {
          (false, false) => {
            map.insert(Case::GenSg, from_stem_or_lemma(vec!["ūs"]));
            map.insert(Case::DatSg, from_stem_or_lemma(vec!["uī"]));
            map.insert(Case::AccSg, from_stem_or_lemma(vec!["um"]));
            map.insert(Case::AblSg, from_stem_or_lemma(vec!["ū"]));
            map.insert(Case::VocSg, from_stem_or_lemma(vec!["us"]));
            map.insert(Case::NomPl, from_stem_or_lemma(vec!["ūs"]));
            map.insert(Case::GenPl, from_stem_or_lemma(vec!["uum"]));
            map.insert(Case::DatPl, from_stem_or_lemma(vec!["ibus"]));
            map.insert(Case::AccPl, from_stem_or_lemma(vec!["ūs"]));
            map.insert(Case::AblPl, from_stem_or_lemma(vec!["ibus"]));
            map.insert(Case::VocPl, from_stem_or_lemma(vec!["ūs"]));
          },
          (true, false) => {
            map.insert(Case::GenSg, from_stem_or_lemma(vec!["ūs", "ū"]));
            map.insert(Case::DatSg, from_stem_or_lemma(vec!["ūī", "ū"]));
            map.insert(Case::AccSg, from_stem_or_lemma(vec!["ū"]));
            map.insert(Case::AblSg, from_stem_or_lemma(vec!["ū"]));
            map.insert(Case::VocSg, from_stem_or_lemma(vec!["ū"]));
            map.insert(Case::NomPl, from_stem_or_lemma(vec!["ua"]));
            map.insert(Case::GenPl, from_stem_or_lemma(vec!["uum"]));
            map.insert(Case::DatPl, from_stem_or_lemma(vec!["ibus"]));
            map.insert(Case::AccPl, from_stem_or_lemma(vec!["ua"]));
            map.insert(Case::AblPl, from_stem_or_lemma(vec!["ibus"]));
            map.insert(Case::VocPl, from_stem_or_lemma(vec!["ua"]));
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
      },
      Self::D5 { ies } => {
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
        map.insert(Case::NomPl, from_stem_or_lemma(vec!["ēs"]));
        map.insert(Case::GenPl, from_stem_or_lemma(vec!["ērum"]));
        map.insert(Case::DatPl, from_stem_or_lemma(vec!["ēbus"]));
        map.insert(Case::AccPl, from_stem_or_lemma(vec!["ēs"]));
        map.insert(Case::AblPl, from_stem_or_lemma(vec!["ēbus"]));
        map.insert(Case::VocPl, from_stem_or_lemma(vec!["ēs"]));
        map.insert(Case::LocPl, from_stem_or_lemma(vec!["ēbus"]));
      },
      _ => {},
    }

    map
  }
}

fn latina(word: &str, request: Request) -> TemplateText {
  let noun = Noun::new(word);

  let mut subwords = Vec::new();
  if let Ok(word) = env_var("ENV_f") { subwords.push(word); }
  if let Ok(word) = env_var("ENV_m") { subwords.push(word); }
  let tags = noun.tags.into_iter().map(|v| v.to_string()).collect();
  let notes = noun.footnote;
  let conjugation = None;
  let pronunciation = None;
  let meanings = None;
  let examples = None;

  match request {
    Request::Lemma => TemplateText {
      mutation: None,
      lemma: Some(noun.lemmas[0].clone()),
      declension: None,
      notes,
      subwords,
      tags,
      conjugation,
      pronunciation,
      meanings,
      examples,
    },
    Request::Table => TemplateText {
      mutation: Some(noun.table),
      lemma: None,
      declension: Some(noun.declension),
      notes,
      subwords,
      tags,
      conjugation,
      pronunciation,
      meanings,
      examples,
    },
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

enum Request {
  Lemma,
  Table,
}
