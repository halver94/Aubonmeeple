use std::collections::HashSet;

use regex::Regex;
use unidecode::unidecode;

static TOKENS_UNWANTED: [&str; 23] = [
    "vf",
    "vo",
    "edition",
    "et",
    "le",
    "jeu",
    "de",
    "societe",
    "plateau",
    "boardgame",
    "board",
    "game",
    "the",
    "game",
    "base",
    "expansion",
    "deck",
    "deckbuiding",
    "buiding",
    "d",
    "of",
    "boite",
    "box",
];

static CHAR_UNWANTED: [&str; 11] = [":", "-", "\'", "&", "[", "]", "=", ",", "!", "`", "’"];

pub fn clean_name(name: &str) -> String {
    log::trace!("cleaning name : {}", name);
    let re = Regex::new(&CHAR_UNWANTED.join("|")).unwrap();
    let name_cleaned = re.replace_all(name, " ").to_string();
    log::trace!("cleaned name : {}", name_cleaned);
    name_cleaned
}

pub fn are_names_similar(name1: &str, name2: &str) -> bool {
    log::trace!("name1 : {}, name2 : {}", name1, name2);
    let name1_clean = clean_name(name1);
    let name2_clean = clean_name(name2);
    log::trace!(
        "name1_clean : {}, name2_clean : {}",
        name1_clean,
        name2_clean
    );

    let words_name1: HashSet<String> = name1_clean
        .split_whitespace()
        .map(|word| unidecode(word).to_lowercase())
        .collect();
    let words_name2: HashSet<String> = name2_clean
        .split_whitespace()
        .map(|word| unidecode(word).to_lowercase())
        .collect();
    log::trace!(
        "words_name1 : {:#?}, words_name2 : {:#?}",
        words_name1,
        words_name2
    );

    let difference = words_name1.symmetric_difference(&words_name2);
    log::trace!("difference raw: {:#?}", difference);

    let difference_tokens_unwanted: Vec<String> = difference
        .map(|word| unidecode(word).to_lowercase())
        .filter(|word| !TOKENS_UNWANTED.contains(&word.as_str()))
        .collect();

    if !difference_tokens_unwanted.is_empty() {
        for word in difference_tokens_unwanted {
            log::trace!("Word not in tokens_unwanted: {}", word);
        }
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::website::helper::are_names_similar;

    struct Test<'a> {
        name1: &'a str,
        name2: &'a str,
        result: bool,
    }

    #[test]
    fn test_parsing() {
        let tests = vec![
            Test {
                name1: "Break In - Tour Eiffel",
                name2: "Break In : Tour Eiffel",
                result: true,
            },
            Test {
                name1: "Break\n                In - Tour Eiffel",
                name2: "Break In : Tour Eiffel",
                result: true,
            },
            Test {
                name1: "Death Note - le jeu d'enquete",
                name2: "Death Note Le Jeu D'enquête",
                result: true,
            },
            Test {
                name1: "Quarto Mini",
                name2: "Quarto! Mini",
                result: true,
            },
            Test {
                name1: "Les Flammes d’Adlerstein",
                name2: "Les Flammes D'adlerstein",
                result: true,
            },
            Test {
                name1: "Tiny Epic Western Base",
                name2: "Tiny Epic Western",
                result: true,
            },
            Test {
                name1: "Strife: Shadows & Steam",
                name2: "Strife Shadows Steam",
                result: true,
            },
            Test {
                name1: "It's a Wonderful World: Corruption & Ascension",
                name2: "It s A Wonderful World Corruption Ascension",
                result: true,
            },
            Test {
                name1: "Skaal",
                name2: "Skåål",
                result: true,
            },
        ];
        for test in tests.into_iter() {
            assert_eq!(are_names_similar(&test.name1, &test.name2), test.result);
        }
    }
}
