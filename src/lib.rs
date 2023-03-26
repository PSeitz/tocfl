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
    pub tocfl_level: u64,
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

pub struct Dictionary {
    pub hashmap: HashMap<(String, String), Entry>,
}
impl Dictionary {
    /// Get an entry for its traditional + pinyin combination
    pub fn get_entry(&self, traditional: &str, pinyin: &str) -> Option<&Entry> {
        self.hashmap
            .get(&(traditional.to_string(), pinyin.to_string()))
    }

    /// Iterator over all entries
    pub fn iter(&self) -> impl Iterator<Item = &Entry> + '_ {
        self.hashmap.values()
    }
}

pub fn get_entries() -> HashMap<(String, String), Entry> {
    let rows = include_str!("../tocfl_words.json");
    rows.lines()
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
                .map(move |(chin, pin)| ((chin.to_string(), pin.to_string()), entry.clone()))
        })
        .collect()
}

#[test]
fn entry_test1() {
    get_entries()
        .get(&("爸爸".to_string(), "bàba".to_string()))
        .unwrap();
}

#[test]
fn entry_test2() {
    get_entries()
        .get(&("爸".to_string(), "bà".to_string()))
        .unwrap();
}
