use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use lazy_static::lazy_static;
use rusqlite::Connection;

lazy_static! {
    static ref DIAGRAMS: HashSet<String> =
        include_str!("../data/tw3diagramlist.txt").trim().lines().map(slugify).collect();
    static ref FORMULAE: HashSet<String> =
        include_str!("../data/tw3formulaelist.txt").trim().lines().map(slugify).collect();
    static ref QUESTS: HashSet<String> =
        include_str!("../data/tw3questlist.txt").trim().lines().map(slugify).collect();
    static ref DEFAULT_DIAGRAMS: HashSet<String> =
        include_str!("../data/tw3defaultdiagramlist.txt").trim().lines().map(slugify).collect();
    static ref DEFAULT_FORMULAE: HashSet<String> =
        include_str!("../data/tw3defaultformulaelist.txt").trim().lines().map(slugify).collect();
}

pub struct GameRun {
    conn: Connection,
}

impl GameRun {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("tw3hundo.db")?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS diagrams (
                diagram TEXT NOT NULL UNIQUE,
                found INT DEFAULT 0
            )
            "#,
            (),
        )?;
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS formulae (
                formula TEXT NOT NULL UNIQUE,
                found INT DEFAULT 0
            )
            "#,
            (),
        )?;
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS quests (
                quest TEXT NOT NULL UNIQUE,
                found INT DEFAULT 0
            )
            "#,
            (),
        )?;
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS logs (
                logtime TEXT,
                message TEXT,
                content TEXT
            )
            "#,
            (),
        )?;
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS perf (
                timing REAL
            )
            "#,
            (),
        )?;

        conn.execute("BEGIN TRANSACTION;", ())?;

        {
            let mut stmt =
                conn.prepare("INSERT OR IGNORE INTO diagrams (diagram, found) VALUES (?, 1)")?;
            for diagram in &*DEFAULT_DIAGRAMS {
                stmt.execute((diagram,))?;
            }
        }

        {
            let mut stmt =
                conn.prepare("INSERT OR IGNORE INTO formulae (formula, found) VALUES (?, 1)")?;
            for formula in &*DEFAULT_FORMULAE {
                stmt.execute((formula,))?;
            }
        }

        {
            let mut stmt = conn.prepare("INSERT OR IGNORE INTO diagrams (diagram) VALUES (?)")?;
            for diagram in &*DIAGRAMS {
                stmt.execute((diagram,))?;
            }
        }

        {
            let mut stmt = conn.prepare("INSERT OR IGNORE INTO formulae (formula) VALUES (?)")?;
            for formula in &*FORMULAE {
                stmt.execute((formula,))?;
            }
        }

        {
            let mut stmt = conn.prepare("INSERT OR IGNORE INTO quests (quest) VALUES (?)")?;
            for quest in &*QUESTS {
                stmt.execute((quest,))?;
            }
        }

        conn.execute("COMMIT;", ())?;

        Ok(Self { conn })
    }

    pub fn log<S: AsRef<str>, T: AsRef<str>>(&mut self, message: S, content: T) -> Result<()> {
        tracing::info!("{}: {}", message.as_ref(), content.as_ref());
        self.conn.execute(
            "INSERT INTO logs (logtime, message, content) VALUES (datetime(), ?, ?)",
            [message.as_ref(), content.as_ref()],
        )?;
        Ok(())
    }

    pub fn timing(&mut self, time: Duration) -> Result<()> {
        self.conn.execute("INSERT INTO perf (timing) VALUES (?)", [time.as_secs_f64()])?;
        Ok(())
    }

    pub fn flag_diagram(&mut self, diagram: &str) -> Result<()> {
        self.conn.execute("UPDATE diagrams SET found = 1 WHERE diagram = ?", [diagram])?;
        self.log("FOUND DIAGRAM", diagram)?;
        Ok(())
    }

    pub fn flag_formula(&mut self, formula: &str) -> Result<()> {
        self.conn.execute("UPDATE formulae SET found = 1 WHERE formula = ?", [formula])?;
        self.log("FOUND FORMULA", formula)?;
        Ok(())
    }

    pub fn flag_quest(&mut self, quest: &str) -> Result<()> {
        self.conn.execute("UPDATE quests SET found = 1 WHERE quest = ?", [quest])?;
        self.log("FOUND QUEST", quest)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quest(String),
    Formula(String),
    Diagram(String),
}

pub fn tokenize<S: AsRef<str>>(s: S) -> Option<Action> {
    fn quest_completed(s: &str) -> Option<Action> {
        let mut it = s.split_whitespace();
        it.find(|&s| s.contains("quest"))?;
        it.find(|&s| s.contains("completed"))?;
        Some(Action::Quest(it.intersperse(" ").collect()))
    }

    fn new_alchemy_formula(s: &str) -> Option<Action> {
        let mut it = s.split_whitespace();
        it.find(|&s| s.contains("new"))?;
        it.find(|&s| s.contains("alchemy"))?;
        it.find(|&s| s.contains("formula"))?;
        Some(Action::Formula(it.intersperse(" ").collect()))
    }

    fn new_crafting_diagram(s: &str) -> Option<Action> {
        let mut it = s.split_whitespace();
        it.find(|&s| s.contains("new"))?;
        it.find(|&s| s.contains("crafting"))?;
        it.find(|&s| s.contains("diagram"))?;
        Some(Action::Diagram(it.intersperse(" ").collect()))
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
            tokenize("4 new alchemy formula s tornout page ancient leshen decoction"),
            Some(Action::Formula("s tornout page ancient leshen decoction".to_string()))
        );
        assert_eq!(
            tokenize("new alchemy formula tornout page ekimmara decoction"),
            Some(Action::Formula("tornout page ekimmara decoction".to_string()))
        );
        assert_eq!(
            tokenize("5 gnew crafting diagram diagrtm broadhead bolt 2 r"),
            Some(Action::Diagram("diagrtm broadhead bolt 2 r".to_string()))
        );
        assert_eq!(
            tokenize("quest completed a frying pan spick and span"),
            Some(Action::Quest("a frying pan spick and span".to_string()))
        );
    }

    #[test]
    fn test_db() {
        GameRun::new().unwrap();
    }
}
