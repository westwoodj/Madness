pub mod espn;
pub mod odds;

use anyhow::Result;
use rusqlite::Connection;

use crate::db::queries;
use espn::EspnClient;

/// Fetch all tournament team stats from ESPN and store in DB.
pub fn fetch_all_stats(conn: &Connection, season: i64) -> Result<()> {
    let client = EspnClient::new()?;
    let teams = queries::get_all_teams(conn)?;

    if teams.is_empty() {
        println!("No teams in DB. Run `madness bracket` first.");
        return Ok(());
    }

    println!("Fetching stats for {} teams (season {})...", teams.len(), season);

    for (i, team) in teams.iter().enumerate() {
        let espn_id = match &team.espn_id {
            Some(id) if !id.is_empty() => id.clone(),
            _ => {
                println!("  [{}/{}] {} — no ESPN ID, skipping", i + 1, teams.len(), team.name);
                continue;
            }
        };

        print!("  [{}/{}] {}...", i + 1, teams.len(), team.name);
        match client.fetch_team_stats(conn, team.id, &espn_id, season) {
            Ok(()) => println!(" ✓"),
            Err(e) => println!(" ✗ ({})", e),
        }

        // Rate limit: ~3 req/s
        std::thread::sleep(std::time::Duration::from_millis(350));
    }

    println!("\nFetching per-game schedules...");
    for (i, team) in teams.iter().enumerate() {
        let espn_id = match &team.espn_id {
            Some(id) if !id.is_empty() => id.clone(),
            _ => continue,
        };

        print!("  [{}/{}] {} schedule...", i + 1, teams.len(), team.name);
        match client.fetch_team_schedule(conn, team.id, &espn_id, season) {
            Ok(n) => println!(" {} games", n),
            Err(e) => println!(" ✗ ({})", e),
        }

        std::thread::sleep(std::time::Duration::from_millis(350));
    }

    // Fetch scoreboard for current tournament dates
    println!("\nFetching tournament scoreboard for odds...");
    let dates = tournament_dates_2026();
    for date in &dates {
        print!("  Scoreboard {}...", date);
        match client.fetch_scoreboard(conn, date) {
            Ok(()) => println!(" ✓"),
            Err(e) => println!(" ✗ ({})", e),
        }
        std::thread::sleep(std::time::Duration::from_millis(350));
    }

    println!("\nDone.");
    Ok(())
}

fn tournament_dates_2026() -> Vec<String> {
    // First Four: Mar 17-18, Round of 64: Mar 19-20, Round of 32: Mar 21-22
    // Sweet 16: Mar 26-27, Elite 8: Mar 28-29, Final 4: Apr 4, Championship: Apr 6
    vec![
        "20260317", "20260318", "20260319", "20260320",
        "20260321", "20260322", "20260326", "20260327",
        "20260328", "20260329", "20260404", "20260406",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
