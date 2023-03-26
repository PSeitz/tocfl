use std::io::Write;

use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};
use tocfl::Entry;

fn main() -> Result<(), Error> {
    convert(
        "Vocabulary_List_111-11-14.xlsx",
        "總詞表", // This sheet contains all information
        "tocfl_words.json",
    )?;
    Ok(())
}

fn get_comps(text: &str) -> Vec<String> {
    text.split("/")
        .map(|el| el.trim())
        .map(ToOwned::to_owned)
        .collect()
}
fn convert(file: &str, workbook_name: &str, out_file: &str) -> Result<(), Error> {
    let mut workbook: Xlsx<_> = open_workbook(file)?;
    let range = workbook
        .worksheet_range(workbook_name)
        .ok_or(Error::Msg("Cannot find workbook"))??;

    let mut iter = RangeDeserializerBuilder::new().from_range(&range)?.skip(2);
    let mut fs = std::fs::File::create(out_file).unwrap();

    let mut rows = Vec::new();
    while let Some(Ok(result)) = iter.next() {
        // Sample row
        // 1	愛	基礎	第1級	核心詞	535	681	[['愛', ['39542']]]	ㄞˋ	ài
        let val: (
            u64,
            String,
            String,
            String,
            String,
            u64,
            u64,
            String,
            String,
            String,
        ) = result;
        let vocabs: Vec<String> = get_comps(&val.1);
        let zhuyins: Vec<String> = get_comps(&val.8);
        let pinyins: Vec<String> = get_comps(&val.9);
        let row = Entry {
            id: val.0,
            text: vocabs[0].to_string(),
            text_alt: vocabs[1..].to_vec(),
            category: val.2,
            tocfl_level: remove_non_digits(&val.3).unwrap(),
            situation: val.4,

            written_per_million: val.5,
            spoken_per_million: val.6,

            components: val.7,

            zhuyin: zhuyins[0].to_string(),
            zhuyin_alt: zhuyins[1..].to_vec(),

            pinyin: pinyins[0].to_string(),
            pinyin_alt: pinyins[1..].to_vec(),
        };
        rows.push(row);
    }
    normalize(&mut rows);

    for row in &rows {
        fs.write_all(serde_json::to_string(&row).unwrap().as_bytes())
            .unwrap();
        fs.write_all(b"\n").unwrap();
    }

    Ok(())
}

fn remove_non_digits(input: &str) -> Option<u64> {
    let digits: String = input.chars().filter(|c| c.is_digit(10)).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u64>().ok()
    }
}

// normalization

// Some entries have multiple  entries with a number
//
// 9281	著1	精熟	第6級		4017	1760	[['著', ['29560', '29632', '29651', '30743', '30817']]]	ㄓㄨˋ	zhù
// 9347	著2	精熟	第6級		4017	1760	[['著', ['29560', '29632', '29651', '30743', '30817']]]	ㄓㄨㄛˊ	zhuó
// 14061	著3	精熟	第7級		4017	1760	[['著', ['29560', '29632', '29651', '30743', '30817']]]	ㄓㄠˊ	Y"和
fn normalize(rows: &mut Vec<Entry>) {
    let average_written_per_million_by_tocfl_level =
        average_written_per_million_by_tocfl_level(&rows);

    // some entries have a number after the character.
    // We don't want that number
    //
    // Those entries usually have a wrong frequency assigned.
    // We fix that, by using the average frequency on that level instead
    //
    for row in rows {
        let (stripped_text, has_digit) = remove_digits(&row.text);
        if has_digit {
            row.text = stripped_text;
            let new_freq = average_written_per_million_by_tocfl_level[row.tocfl_level as usize];
            row.spoken_per_million = new_freq;
            row.written_per_million = new_freq;
        }
    }
}

fn remove_digits(string: &str) -> (String, bool) {
    let mut has_digit = false;
    // filter out digits from the string
    let filtered_chars: String = string
        .chars()
        .filter(|c| {
            if c.is_digit(10) {
                has_digit = true;
                false
            } else {
                true
            }
        })
        .collect();
    // return the filtered string and whether there was a digit
    (filtered_chars, has_digit)
}

fn average_written_per_million_by_tocfl_level(rows: &[Entry]) -> Vec<u64> {
    let mut sum_written_per_million = vec![0; 8];
    let mut count_by_tocfl_level = vec![0; 8];

    for row in rows {
        let level = row.tocfl_level as usize;
        if level >= 1 && level <= 7 {
            sum_written_per_million[level] += row.written_per_million;
            count_by_tocfl_level[level] += 1;
        }
    }

    sum_written_per_million
        .iter()
        .zip(count_by_tocfl_level.iter())
        .map(|(&sum, &count)| {
            if count > 0 {
                (sum as f64 / count as f64) as u64
            } else {
                0
            }
        })
        .collect()
}
