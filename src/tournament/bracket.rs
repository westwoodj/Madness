use anyhow::Result;
use rusqlite::Connection;

use crate::db::{
    models::{Matchup, Team},
    queries,
};
use crate::tournament::seed_2026;

/// Seed the database with the 2026 tournament bracket.
/// Inserts all 68 teams and generates Round of 64 matchups.
pub fn seed_bracket(conn: &Connection) -> Result<()> {
    let teams = seed_2026::teams_2026();
    let pairings = seed_2026::r64_pairings();
    let regions = ["East", "West", "South", "Midwest"];

    // Insert all teams
    for (seed, region, name, abbrev, espn_id) in &teams {
        let team = Team {
            id: 0,
            name: name.to_string(),
            abbrev: Some(abbrev.to_string()),
            seed: Some(*seed),
            region: Some(region.to_string()),
            conference: None,
            espn_id: Some(espn_id.to_string()),
        };
        queries::upsert_team(conn, &team)?;
    }

    println!("Inserted {} teams.", teams.len());

    // Generate Round of 64 matchups (round = 1)
    // First clear any existing R64 matchups to avoid duplicates
    conn.execute("DELETE FROM matchups WHERE round = 1", [])?;

    let mut matchup_count = 0;
    for region in &regions {
        // Collect teams in this region by seed
        let region_teams: Vec<_> = teams
            .iter()
            .filter(|(_, r, _, _, _)| r == region)
            .collect();

        for (seed1, seed2) in &pairings {
            let t1 = region_teams.iter().find(|(s, _, _, _, _)| s == seed1);
            let t2 = region_teams.iter().find(|(s, _, _, _, _)| s == seed2);

            if let (Some((_, _, name1, _, _)), Some((_, _, name2, _, _))) = (t1, t2) {
                let team1 = queries::get_team_by_name(conn, name1)?;
                let team2 = queries::get_team_by_name(conn, name2)?;

                if let (Some(t1), Some(t2)) = (team1, team2) {
                    let matchup = Matchup {
                        id: 0,
                        round: 1,
                        region: Some(region.to_string()),
                        team1_id: Some(t1.id),
                        team2_id: Some(t2.id),
                        team1_ml: None,
                        team2_ml: None,
                        spread: None,
                        over_under: None,
                        winner_id: None,
                        game_date: None,
                    };
                    queries::insert_matchup(conn, &matchup)?;
                    matchup_count += 1;
                }
            }
        }
    }

    println!("Created {} Round of 64 matchups.", matchup_count);
    Ok(())
}

/// Returns teams grouped by region, sorted by seed (for bracket display)
pub fn teams_by_region(teams: &[Team]) -> Vec<(String, Vec<&Team>)> {
    let region_order = ["East", "West", "South", "Midwest"];
    region_order
        .iter()
        .map(|region| {
            let mut region_teams: Vec<&Team> = teams
                .iter()
                .filter(|t| t.region.as_deref() == Some(region))
                .collect();
            region_teams.sort_by_key(|t| t.seed.unwrap_or(99));
            (region.to_string(), region_teams)
        })
        .collect()
}
