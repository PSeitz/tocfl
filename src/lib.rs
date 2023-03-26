use prettify_pinyin::prettify;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    let normalized = remove_whitespace(normalized);
    normalized
}

impl<V> TOCFLDictionary<V> {
    /// Get an entry for its traditional + pinyin combination
    pub fn get_entry(&self, traditional: &str, pinyin: &str) -> Option<&V> {
        self.hashmap
            .get(&(traditional.to_string(), normalize_pinyin(pinyin)))
            //fallback remove pinyin
            .or(self.hashmap.get(&(traditional.to_string(), "".to_string())))
    }

    /// Iterator over all entries
    pub fn iter(&self) -> impl Iterator<Item = &V> + '_ {
        self.hashmap.values()
    }
}

/// Compile a hashmap of `HashMap<(Char, Pinyin), CountPerMillion>` by building a commonness HashMap of chars from words
///
/// Those chars may not be common as themselves
pub fn compile_common_chars() -> TOCFLDictionary<u64> {
    let dict = load_tocfl_dictionary();

    let hashmap = dict.hashmap;

    let mut cha_to_pinyin = HashMap::new();
    for (word, pinyin) in hashmap.keys() {
        if word.chars().count() != 1 {
            continue;
        }
        for cha in word.chars() {
            cha_to_pinyin.entry(cha).or_insert(pinyin.to_string());
        }
    }

    // We add the word parts to the chars, although single chars may be not that common
    // e.g. 午 on its own is uncommon, but 下午 [xiawu] is quite common
    let mut char_hash_map = HashMap::new();
    let empty_fall_back = "".to_string();
    for ((word, _pinyin), v) in hashmap.iter() {
        if word.chars().count() <= 1 {
            continue;
        }
        // TODO tokenize _pinyin and use that would be better
        for cha in word.chars() {
            let pinyin = cha_to_pinyin.get(&cha).unwrap_or(&empty_fall_back);
            let key = (cha.to_string(), remove_whitespace(pinyin.to_string()));
            let entry = char_hash_map.entry(key).or_insert_with(Default::default);
            *entry += v.written_per_million;
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
            let mut first = vec![(entry.text.to_string(), entry.pinyin.to_string())];
            let other = entry
                .text_alt
                .iter()
                .map(ToString::to_string)
                .zip(entry.pinyin_alt.iter().map(ToString::to_string));
            first.extend(other);
            first
                .into_iter()
                .map(move |(chin, pin)| ((chin.to_string(), remove_whitespace(pin)), entry.clone()))
        })
        .collect();

    TOCFLDictionary { hashmap }
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
fn entry_test3() {
    load_tocfl_dictionary().get_entry("爸", "bà").unwrap();
}

#[test]
fn entry_test4() {
    load_tocfl_dictionary()
        .get_entry("安靜", "ān jìng")
        .unwrap();
}
