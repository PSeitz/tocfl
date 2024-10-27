use pinyin::ToPinyin;
use prettify_pinyin::prettify;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Ordinal
    pub id: u64,
    /// Traditional Chinese
    pub text: String,
    /// Other Variants
    pub text_alt: Vec<String>,
    /// Category
    /// Base, advanced etc. in Chinese, e.g. 基礎
    pub category: String,
    /// TOCFL Level, 1-7
    pub tocfl_level: u32,
    /// Situation the word is used
    pub situation: String,

    /// Count per million written
    pub written_per_million: u64,
    /// Count per million spoken
    pub spoken_per_million: u64,

    /// No idea what that is.
    /// e.g. [['的', ['7457', '8310', '8568']]]
    pub components: String,

    /// Zhuyin
    pub zhuyin: String,
    /// Pinyin Other Variants
    pub zhuyin_alt: Vec<String>,

    /// Pinyin
    pub pinyin: String,
    /// Pinyin Other Variants
    pub pinyin_alt: Vec<String>,
}

pub struct TOCFLDictionary<V> {
    pub hashmap: HashMap<(String, String), V>,
}

fn remove_whitespace(mut s: String) -> String {
    s.retain(|c| !c.is_whitespace());
    s
}

fn normalize_pinyin(pinyin: &str) -> String {
    let normalized: String = prettify(pinyin.to_string());

    remove_whitespace(normalized)
}

impl<V> TOCFLDictionary<V> {
    /// Get an entry for its traditional chinese character + pinyin combination
    /// Prefer to use this to differentiate between different characters that have multiple pronounciations
    ///
    /// Note that some characters have multiple pronounciations, e.g. 分 fēn and fèn
    ///
    /// The pinyin can have the format "yì" or "yi4"
    ///
    /// # Limitation
    /// Note that some characters don't have a pinyin, e.g. 食.
    ///
    pub fn get_entry(&self, traditional: &str, pinyin: &str) -> Option<&V> {
        self.hashmap
            .get(&(traditional.to_string(), normalize_pinyin(pinyin)))
    }

    /// Get an entry for its traditional chinese character
    pub fn get_entry_no_pinyin(&self, traditional: &str) -> Option<&V> {
        self.hashmap.get(&(traditional.to_string(), "".to_string()))
    }

    /// Get an entry for its traditional + [&pinyin] combination
    pub fn get_entry_multiple(&self, traditional: &str, pinyin: &[&str]) -> Option<&V> {
        for pinyin in pinyin {
            if let Some(entry) = self
                .hashmap
                .get(&(traditional.to_string(), normalize_pinyin(pinyin)))
            {
                return Some(entry);
            }
        }
        //fallback remove pinyin
        self.hashmap.get(&(traditional.to_string(), "".to_string()))
    }

    /// Iterator over all entries
    pub fn iter(&self) -> impl Iterator<Item = &V> + '_ {
        self.hashmap.values()
    }
}

/// Compile a hashmap of `HashMap<(Char, Pinyin), CountPerMillion>` by building a commonness HashMap of chars from words
///
/// Those chars may not be common themselves and may be more common in words
pub fn compile_common_chars() -> TOCFLDictionary<u64> {
    let dict = load_tocfl_dictionary();

    let hashmap = dict.hashmap;

    // Note that we only add pinyin if there is only on character
    let mut cha_to_pinyin: HashMap<char, Vec<String>> = HashMap::new();
    for (word, pinyin) in hashmap.keys() {
        if word.chars().count() != 1 {
            continue;
        }
        for cha in word.chars() {
            let pinyins = cha_to_pinyin.entry(cha).or_default();

            if pinyin.trim().is_empty() {
                continue;
            }
            pinyins.push(pinyin.to_string());
        }
    }

    // We add the word parts to the chars, although single chars may be not that common
    // e.g. 午 on its own is uncommon, but 下午 [xiawu] is quite common
    let mut char_hash_map = HashMap::new();
    let empty_fall_back = vec![];
    for ((word, _pinyin), v) in hashmap.iter() {
        if word.chars().count() <= 1 {
            continue;
        }
        let mut add_entry = |cha: char, pinyin: &str| {
            let key = (cha.to_string(), remove_whitespace(pinyin.to_string()));
            let entry = char_hash_map.entry(key).or_insert_with(Default::default);
            *entry += v.written_per_million;
        };
        // TODO tokenize _pinyin and use that would be better
        for cha in word.chars() {
            let pinyin = cha_to_pinyin.get(&cha).unwrap_or(&empty_fall_back);

            if pinyin.len() == 1 {
                let pinyin = &pinyin[0];
                add_entry(cha, &remove_whitespace(pinyin.to_string()));

                // Add empty fallback
                add_entry(cha, "");
            }
            if pinyin.is_empty() {
                // Add empty fallback
                add_entry(cha, "");
                // Add default from character to pinyin conversion
                if let Some(pinyin) = cha.to_pinyin() {
                    add_entry(cha, pinyin.with_tone());
                }
            }
        }
    }

    TOCFLDictionary {
        hashmap: char_hash_map,
    }
}

pub fn load_tocfl_dictionary() -> TOCFLDictionary<Entry> {
    let rows = include_str!("../tocfl_words.json");
    let hashmap: HashMap<(String, String), Entry> = rows
        .lines()
        .flat_map(|line| {
            let entry: Entry = serde_json::from_str(line).unwrap();
            let mut first_and_pinyin_fallback = vec![
                (entry.text.to_string(), entry.pinyin.to_string()),
                (entry.text.to_string(), "".to_string()),
            ];
            let other = entry
                .text_alt
                .iter()
                .map(ToString::to_string)
                .zip(entry.pinyin_alt.iter().map(ToString::to_string));
            first_and_pinyin_fallback.extend(other);
            first_and_pinyin_fallback
                .into_iter()
                .map(move |(chin, pin)| ((chin.to_string(), remove_whitespace(pin)), entry.clone()))
        })
        .collect();

    TOCFLDictionary { hashmap }
}
#[test]
fn test_normalize() {
    assert_eq!(normalize_pinyin("yì shì"), "yìshì");
    assert_eq!(normalize_pinyin("yi4 shi4"), "yìshì");
    // For that we need a tokenizer
    //assert_eq!(normalize_pinyin("yi4shi4"), "yìshì");
}

#[test]
fn entry_test1() {
    load_tocfl_dictionary().get_entry("爸爸", "bàba").unwrap();
}

#[test]
fn entry_test2() {
    load_tocfl_dictionary().get_entry("爸爸", "bà ba").unwrap();
}

#[test]
fn entry_awareness() {
    //dbg!(load_tocfl_dictionary().get_entry_no_pinyin("意識").unwrap());

    load_tocfl_dictionary().get_entry("意識", "yì shì").unwrap();
    load_tocfl_dictionary().get_entry("意識", "yìshì").unwrap();

    load_tocfl_dictionary()
        .get_entry("意識", "yi4 shi4")
        .unwrap();
    //load_tocfl_dictionary()
    //.get_entry("意識", "yi4shi4")
    //.unwrap();
}

#[test]
fn entry_test3() {
    load_tocfl_dictionary().get_entry("爸", "bà").unwrap();
}

#[test]
fn entry_test4() {
    load_tocfl_dictionary()
        .get_entry("安靜", "ān jìng")
        .unwrap();
}
#[test]
fn entry_test_fen1() {
    load_tocfl_dictionary().get_entry("分", "fēn").unwrap();
    load_tocfl_dictionary().get_entry("分", "fen1").unwrap();
}
#[test]
fn entry_test_pian_yi() {
    dbg!(load_tocfl_dictionary().get_entry_no_pinyin("便宜").unwrap());
}

#[test]
fn entry_test_fen2() {
    assert_eq!(load_tocfl_dictionary().get_entry("分", "fèn"), None);
}

#[test]
fn entry_test_taberu() {
    assert_eq!(compile_common_chars().get_entry_no_pinyin("食"), Some(&712));
    assert_eq!(compile_common_chars().get_entry("食", "shí"), Some(&712));
}

#[test]
fn entry_test_hui_painting() {
    assert_eq!(compile_common_chars().get_entry("繪", "hui4"), Some(&120));
    assert_eq!(compile_common_chars().get_entry_no_pinyin("繪"), Some(&120));
}

#[test]
fn entry_test_hui_meeting() {
    assert_eq!(compile_common_chars().get_entry("會", "hui4"), Some(&3624));
    assert_eq!(
        compile_common_chars().get_entry_no_pinyin("會"),
        Some(&3624)
    );
}
