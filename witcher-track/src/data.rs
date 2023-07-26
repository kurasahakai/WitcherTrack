use std::collections::HashSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref DIAGRAMS: HashSet<String> =
        include_str!("../data/tw3diagramlist.txt").trim().lines().map(text_preprocess).collect();
    static ref FORMULAE: HashSet<String> =
        include_str!("../data/tw3formulaelist.txt").trim().lines().map(text_preprocess).collect();
    static ref QUESTS: HashSet<String> =
        include_str!("../data/tw3questlist.txt").trim().lines().map(text_preprocess).collect();
    static ref DEFAULT_DIAGRAMS: HashSet<String> =
        include_str!("../data/tw3defaultdiagramlist.txt")
            .trim()
            .lines()
            .map(text_preprocess)
            .collect();
    static ref DEFAULT_FORMULAE: HashSet<String> =
        include_str!("../data/tw3defaultformulaelist.txt")
            .trim()
            .lines()
            .map(text_preprocess)
            .collect();
}

pub fn text_preprocess<S: Into<String>>(s: S) -> String {
    s.into()
        .to_lowercase()
        .chars()
        .filter(|&char| match char {
            char if char.is_ascii_alphanumeric() => true,
            ' ' => true,
            _ => false,
        })
        .collect::<String>()
        .split_whitespace()
        .intersperse(" ")
        .collect::<String>()
}

#[test]
fn test_quests() {
    println!("{:#?}", *DIAGRAMS);
    println!("{:#?}", *FORMULAE);
    println!("{:#?}", *QUESTS);
    println!(
        "{}+{}={} diagrams",
        DEFAULT_DIAGRAMS.len(),
        DIAGRAMS.len(),
        DEFAULT_DIAGRAMS.union(&*DIAGRAMS).collect::<HashSet<_>>().len()
    );
    println!(
        "{}+{}={} formulae",
        DEFAULT_FORMULAE.len(),
        FORMULAE.len(),
        DEFAULT_FORMULAE.union(&*FORMULAE).collect::<HashSet<_>>().len()
    );
    println!("{} quests", QUESTS.len());
}
