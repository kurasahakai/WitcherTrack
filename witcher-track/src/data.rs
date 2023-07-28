use std::collections::HashSet;
use std::str::Lines;

use lazy_static::lazy_static;
use strsim::normalized_damerau_levenshtein;

use crate::STRSIM_THRESHOLD;

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

enum ActionType {
    Quest,
    Formula,
    Diagram,
}

fn check_str(a: &str, b: &str) -> bool {
    normalized_damerau_levenshtein(a, b) > 0.7
}

fn find_action(it: &mut Lines) -> Option<ActionType> {
    it.find_map(|line| {
        let line = slugify(line);

        if check_str(&line, "quest completed") {
            Some(ActionType::Quest)
        } else if check_str(&line, "new alchemy formula") {
            Some(ActionType::Formula)
        } else if check_str(&line, "new crafting diagram") {
            Some(ActionType::Diagram)
        } else {
            None
        }
    })
}

pub fn parse_action<S: AsRef<str>>(s: S) -> Option<Action> {
    let mut lines = s.as_ref().trim().lines();
    let action = find_action(&mut lines)?;
    let target = slugify(lines.next()?);

    match action {
        ActionType::Quest => get_closest_match(&target, &*QUESTS).map(Action::Quest),
        ActionType::Formula => get_closest_match(&target, &*FORMULAE).map(Action::Formula),
        ActionType::Diagram => get_closest_match(&target, &*DIAGRAMS).map(Action::Diagram),
    }
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

fn get_closest_match<'a, I>(word: &str, possibilities: I) -> Option<String>
where
    I: IntoIterator<Item = &'a String>,
{
    let mut matches_with_scores: Vec<(&String, f64)> = possibilities
        .into_iter()
        .filter_map(|possibility| {
            let score = normalized_damerau_levenshtein(word, possibility);
            if score >= STRSIM_THRESHOLD {
                Some((possibility, score))
            } else {
                None
            }
        })
        .collect();

    matches_with_scores.sort_by(|(_, score1), (_, score2)| score2.partial_cmp(score1).unwrap());

    matches_with_scores.into_iter().map(|(matched_word, _)| matched_word.to_string()).next()
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
            Some(Action::Formula("torn out page ancient leshen decoction".to_string()))
        );
        assert_eq!(
            parse_action("new alchemy formula tornout page ekimmara decoction"),
            Some(Action::Formula("torn out page ekimmara decoction".to_string()))
        );
        // assert_eq!(
        //     parse_action("5 gnew crafting diagram diagrtm broadhead bolt 2 r"),
        //     Some(Action::Diagram("diagrtm broadhead bolt 2 r".to_string()))
        // );
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
