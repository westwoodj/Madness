pub mod models;
pub mod queries;
pub mod schema;

use anyhow::Result;
use rusqlite::Connection;
use schema::*;

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    Ok(conn)
}

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch(&format!(
        "{CREATE_TEAMS}\n{CREATE_TEAM_SEASON_STATS}\n{CREATE_TEAM_GAME_STATS}\n\
         {CREATE_GLOBAL_METRICS}\n{CREATE_MATCHUPS}\n{CREATE_PREDICTIONS}"
    ))?;
    // Idempotent column additions for databases created before these columns existed.
    // SQLite returns an error when the column already exists; we intentionally ignore it.
    let _ = conn.execute(ADD_MATCHUP_NEXT_GAME_ID, []);
    let _ = conn.execute(ADD_MATCHUP_NEXT_SLOT, []);
    Ok(())
}
