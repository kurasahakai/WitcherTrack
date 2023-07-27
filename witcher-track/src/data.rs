use std::collections::HashSet;

use lazy_static::lazy_static;
use strsim::normalized_damerau_levenshtein;

lazy_static! {
    pub static ref DIAGRAMS: HashSet<String> =
        include_str!("../data/tw3diagramlist.txt").trim().lines().map(slugify).collect();
    pub static ref FORMULAE: HashSet<String> =
        include_str!("../data/tw3formulaelist.txt").trim().lines().map(slugify).collect();
    pub static ref QUESTS: HashSet<String> =
        include_str!("../data/tw3questlist.txt").trim().lines().map(slugify).collect();
    pub static ref DEFAULT_DIAGRAMS: HashSet<String> =
        include_str!("../data/tw3defaultdiagramlist.txt").trim().lines().map(slugify).collect();
    pub static ref DEFAULT_FORMULAE: HashSet<String> =
        include_str!("../data/tw3defaultformulaelist.txt").trim().lines().map(slugify).collect();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quest(String),
    Formula(String),
    Diagram(String),
}

pub fn parse_action<S: AsRef<str>>(s: S) -> Option<Action> {
    const THRESHOLD: f64 = 0.7;

    fn quest_completed(s: &str) -> Option<Action> {
        let mut it = s.split_whitespace();
        it.find(|&s| s.contains("quest"))?;
        it.find(|&s| s.contains("completed"))?;
        get_closest_match(&it.intersperse(" ").collect::<String>(), &*QUESTS, THRESHOLD)
            .map(Action::Quest)
    }

    fn new_alchemy_formula(s: &str) -> Option<Action> {
        let mut it = s.split_whitespace();
        it.find(|&s| s.contains("new"))?;
        it.find(|&s| s.contains("alchemy"))?;
        it.find(|&s| s.contains("formula"))?;
        get_closest_match(&it.intersperse(" ").collect::<String>(), &*FORMULAE, THRESHOLD)
            .map(Action::Formula)
    }

    fn new_crafting_diagram(s: &str) -> Option<Action> {
        let mut it = s.split_whitespace();
        it.find(|&s| s.contains("new"))?;
        it.find(|&s| s.contains("crafting"))?;
        it.find(|&s| s.contains("diagram"))?;
        get_closest_match(&it.intersperse(" ").collect::<String>(), &*DIAGRAMS, THRESHOLD)
            .map(Action::Diagram)
    }

    if let Some(q) = quest_completed(s.as_ref()) {
        return Some(q);
    }

    if let Some(q) = new_alchemy_formula(s.as_ref()) {
        return Some(q);
    }

    if let Some(q) = new_crafting_diagram(s.as_ref()) {
        return Some(q);
    }

    None
}

pub fn slugify<S: Into<String>>(s: S) -> String {
    s.into()
        .chars()
        .filter_map(|char| match char {
            char if char.is_ascii_alphanumeric() => Some(char.to_ascii_lowercase()),
            char if char.is_whitespace() => Some(' '),
            char if char.is_ascii_punctuation() => Some(' '),
            _ => None,
        })
        .collect::<String>()
        .split_whitespace()
        .intersperse(" ")
        .collect::<String>()
}

fn get_closest_match<'a, I>(word: &str, possibilities: I, cutoff: f64) -> Option<String>
where
    I: IntoIterator<Item = &'a String>,
{
    let mut matches_with_scores: Vec<(&String, f64)> = possibilities
        .into_iter()
        .map(|possibility| (possibility, normalized_damerau_levenshtein(word, possibility)))
        .collect();

    matches_with_scores.sort_by(|(_, score1), (_, score2)| score2.partial_cmp(score1).unwrap());
    // for k in &matches_with_scores {
    //     println!("{k:?}");
    // }

    matches_with_scores
        .into_iter()
        .take_while(|(_, score)| *score >= cutoff)
        .map(|(matched_word, _)| matched_word.to_string())
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_tokenize() {
        assert_eq!(
            parse_action("4 new alchemy formula s tornout page ancient leshen decoction"),
            Some(Action::Formula("s tornout page ancient leshen decoction".to_string()))
        );
        assert_eq!(
            parse_action("new alchemy formula tornout page ekimmara decoction"),
            Some(Action::Formula("tornout page ekimmara decoction".to_string()))
        );
        assert_eq!(
            parse_action("5 gnew crafting diagram diagrtm broadhead bolt 2 r"),
            Some(Action::Diagram("diagrtm broadhead bolt 2 r".to_string()))
        );
        assert_eq!(
            parse_action("quest completed a frying pan spick and span"),
            Some(Action::Quest("a frying pan spick and span".to_string()))
        );
    }

    #[test]
    fn test_tokenize_2() {
        let cases = &[
            "quest completed a frying pan spick and span",
            "quest completed a frying pan spick and span",
            "quest completed a frying pan spick and span",
            "quest completed a frying pan spick and span",
            "quest completed a frying pan spick and span",
            "quest completed a frying pan spick and span",
            "quest completed afrying pan spick and span",
            "quest completed a frying pan spick and span",
            "g h b sr t ro o k quest completed we iy precious cargo ir nt",
            "guv gtyg 5by g ny os quest completed b a precious cargo i zp r z an",
            "ar o i at wy b bt rt i t i hebri y g z 2 pr b i 5 iar all quest completed t \
             acpreciouscargd r i",
            "quest completed precious cargo",
            "quest completed bon deaths bed",
            "quest completed ion deaths bed",
            "quest completed bon deaths bed",
            "quest completed ion deaths bed",
            "quest completed on deaths bed",
            "quest completed on deaths bed",
            "quest completed a sondeathsbed",
            "quest completed on deaths bed",
            "quest completed on deaths bed",
            "quest completed on deaths bed",
            "lquest completed twisted firestarter s",
            "rquest completed i twisted firestarter",
            "dr at il jf qfbquest completed gtwisted firestarter t",
            "5 j oquest completed k i ji witwisted firestarteriy o",
            "quest completed bcontract devil by the well",
            "f l fquest completed ocontract devil by the well",
            "quest completed temerian valuables",
            "quest completed temerian valuables",
            "quest completed tn l the beast of white orchard",
            "quest completed the beast of white orchard",
            "quest completed lthe beast of white orchard",
            "quest completed scavenger hunt viper school ge",
            "quest completed scavenger hunt viper school ge",
            "quest completed scavenger hunt viper school ge",
            "quest completed scavenger hunt viper school ge",
            "quest completed scavenger hunt viper school ge",
            "quest completed scavenger hunt viper school ge",
            "quest completed scavenger hunt viper school ge",
            "quest completed deserter gold",
            "quest completed deserter gold",
            "quest completed deserter gold",
            "quest completed deserter gold",
            "quest completed deserter gold",
            "quest completed deserter gold",
            "quest completed deserter gold",
            "quest completed deserter gold",
            "quest completed dirty funds s",
            "quest completed dirty funds",
            "quest completed dirty funds",
            "quest completed dirty funds",
            "quest completed dirty funds",
            "quest completed dirty funds",
            "quest completed dirty funds",
            "quest completed dirty funds",
            "quest completed gmissing in action",
        ];

        for case in cases {
            println!("{}   -> {:?}", case, parse_action(case));
        }
    }

    #[test]
    fn test_tokenize_3() {
        println!("{:?}", parse_action("new alchemy formula manuscript page dancing st"));
    }
}
