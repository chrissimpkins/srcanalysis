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

    let mut codepoint_counts: HashMap<u32, u128> = HashMap::new();
    let mut total_chars: u128 = 0;
    let mut ascii_chars: u128 = 0;

    // Walk the directory recursively
    for entry in walkdir::WalkDir::new(directory_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            process_file(
                path,
                &mut codepoint_counts,
                &mut total_chars,
                &mut ascii_chars,
            );
        }
    }

    // Create a vector of (codepoint, count) pairs
    let mut count_vec: Vec<(u32, u128)> = codepoint_counts.into_iter().collect();

    // Sort the vector by count in descending order
    count_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // Print the results
    for (codepoint, count) in count_vec {
        let character = char::from_u32(codepoint).unwrap_or(char::REPLACEMENT_CHARACTER);
        let category = get_character_category(character);
        let is_ascii = codepoint <= 0x7F;
        println!(
            "Character: {}, Codepoint: {:04x}, Category: {:?}, ASCII: {}, Count: {}",
            character, codepoint, category, is_ascii, count
        );
    }

    // Print summary report
    let ascii_percent = (ascii_chars as f64 / total_chars as f64) * 100.0;
    let non_ascii_percent = 100.0 - ascii_percent;
    println!("\nSummary:");
    println!("  ASCII encodings: {} ({:.2}%)", ascii_chars, ascii_percent);
    println!(
        "  Non-ASCII encodings: {} ({:.2}%)",
        total_chars - ascii_chars,
        non_ascii_percent
    );
}

fn process_file(
    path: &Path,
    codepoint_counts: &mut HashMap<u32, u128>,
    total_chars: &mut u128,
    ascii_chars: &mut u128,
) {
    match fs::read_to_string(path) {
        Ok(content) => {
            // Normalize the content using NFC normalization
            let normalized_content = content.nfc().collect::<String>();
            for grapheme in normalized_content.graphemes(true) {
                let c = grapheme.chars().next().unwrap();
                if !c.is_control() {
                    *codepoint_counts.entry(c as u32).or_insert(0) += 1;
                    *total_chars += 1;
                    if (c as u32) <= 0x7F {
                        *ascii_chars += 1;
                    }
                }
            }
        }
        Err(err) => {
            if let std::io::ErrorKind::InvalidData = err.kind() {
                eprintln!("Skipping file with invalid UTF-8: {}", path.display());
            } else {
                eprintln!("Error reading file {}: {}", path.display(), err);
            }
        }
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
