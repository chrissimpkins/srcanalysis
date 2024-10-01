// Copyright 2024 Google, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use unicode_categories::UnicodeCategories;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let directory_path = env::args()
        .nth(1)
        .expect("Error: Missing directory path argument");

    let mut codepoint_counts_by_extension: HashMap<String, HashMap<u32, u128>> = HashMap::new();
    let mut total_chars_by_extension: HashMap<String, u128> = HashMap::new();
    let mut ascii_chars_by_extension: HashMap<String, u128> = HashMap::new();

    // Walk the directory recursively
    for entry in walkdir::WalkDir::new(directory_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let extension = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            process_file(
                path,
                &extension,
                &mut codepoint_counts_by_extension,
                &mut total_chars_by_extension,
                &mut ascii_chars_by_extension,
            );
        }
    }

    for (extension, codepoint_counts) in codepoint_counts_by_extension {
        println!("\nFile Extension: {}", extension);
        let mut count_vec: Vec<(u32, u128)> = codepoint_counts.into_iter().collect();
        count_vec.sort_by(|a, b| b.1.cmp(&a.1));

        for (codepoint, count) in count_vec {
            let character = char::from_u32(codepoint).unwrap_or(char::REPLACEMENT_CHARACTER);
            let category = get_character_category(character);
            let is_ascii = codepoint <= 0x7F;
            println!(
                "Character: {}, Codepoint: {:04x}, Category: {:?}, ASCII: {}, Count: {}",
                character, codepoint, category, is_ascii, count
            );
        }

        let total_chars = total_chars_by_extension.get(&extension).unwrap_or(&0);
        let ascii_chars = ascii_chars_by_extension.get(&extension).unwrap_or(&0);
        let ascii_percent = (*ascii_chars as f64 / *total_chars as f64) * 100.0;
        let non_ascii_percent = 100.0 - ascii_percent;
        println!("\nSummary for .{} files:", extension);
        println!("  ASCII encodings: {} ({:.2}%)", ascii_chars, ascii_percent);
        println!(
            "  Non-ASCII encodings: {} ({:.2}%)",
            total_chars - ascii_chars,
            non_ascii_percent
        );
    }
}

fn process_file(
    path: &Path,
    extension: &str,
    codepoint_counts_by_extension: &mut HashMap<String, HashMap<u32, u128>>,
    total_chars_by_extension: &mut HashMap<String, u128>,
    ascii_chars_by_extension: &mut HashMap<String, u128>,
) {
    match fs::read_to_string(path) {
        Ok(content) => {
            let normalized_content = content.nfc().collect::<String>();
            for grapheme in normalized_content.graphemes(true) {
                let c = grapheme.chars().next().unwrap();
                if !c.is_control() {
                    let codepoint = c as u32;
                    *codepoint_counts_by_extension
                        .entry(extension.to_string())
                        .or_insert_with(HashMap::new)
                        .entry(codepoint)
                        .or_insert(0) += 1;
                    *total_chars_by_extension
                        .entry(extension.to_string())
                        .or_insert(0) += 1;
                    if codepoint <= 0x7F {
                        *ascii_chars_by_extension
                            .entry(extension.to_string())
                            .or_insert(0) += 1;
                    }
                }
            }
        }
        Err(e) => eprintln!("Error reading file {}: {}", path.display(), e),
    }
}

fn get_character_category(c: char) -> &'static str {
    if c.is_letter() || c.is_number() {
        "Alphanumeric"
    } else if c.is_separator_space() {
        "Space"
    } else if c.is_punctuation() {
        "Punctuation"
    } else if c.is_symbol() {
        "Symbol"
    } else {
        "Other"
    }
}
