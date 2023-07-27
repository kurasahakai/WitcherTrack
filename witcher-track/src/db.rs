use std::time::Duration;

use anyhow::Result;
use rusqlite::Connection;

use crate::data::{DEFAULT_DIAGRAMS, DEFAULT_FORMULAE, DIAGRAMS, FORMULAE, QUESTS};

/// Game run database handler.
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
