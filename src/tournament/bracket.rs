use anyhow::Result;
use rusqlite::Connection;

use crate::db::{
    models::{Matchup, Team},
    queries,
};
use crate::tournament::seed_2026;

/// Insert a placeholder game (no teams yet) and return its new row id.
fn insert_placeholder(
    conn: &Connection,
    round: i64,
    region: Option<&str>,
    next_game_id: Option<i64>,
    next_slot: Option<i64>,
) -> Result<i64> {
    let m = Matchup {
        id: 0,
        round,
        region: region.map(|s| s.to_string()),
        team1_id: None,
        team2_id: None,
        team1_ml: None,
        team2_ml: None,
        spread: None,
        over_under: None,
        winner_id: None,
        game_date: None,
        next_game_id,
        next_slot,
    };
    queries::insert_matchup(conn, &m)
}

/// Seed the database with the full 2026 tournament bracket.
///
/// Creates all 63 games across rounds 1–6 with `next_game_id` / `next_slot`
/// links so that `advance_winner` can propagate a team through each round.
///
/// Round 1 games receive the actual seeded teams; rounds 2–6 are placeholder
/// games (team slots start as NULL and are filled in as winners advance).
///
/// Bracket half-bracket pairings:
///   East  E8 winner ┐
///                   ├── Final Four game 1 ──┐
///   West  E8 winner ┘                       │
///                                           ├── Championship
///   South   E8 winner ┐                     │
///                     ├── Final Four game 2 ┘
///   Midwest E8 winner ┘
pub fn seed_bracket(conn: &Connection) -> Result<()> {
    let teams = seed_2026::teams_2026();
    let pairings = seed_2026::r64_pairings();
    let regions = ["East", "West", "South", "Midwest"];

    // ── Insert all 68 teams ───────────────────────────────────────────────────
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

    // ── Clear all existing matchups ───────────────────────────────────────────
    conn.execute("DELETE FROM matchups", [])?;

    // ── Build bracket graph bottom-up so we always know the next-round ID ────

    // Round 6: Championship (1 game)
    let champ = insert_placeholder(conn, 6, None, None, None)?;

    // Round 5: Final Four (2 games)
    //   game 1: East vs West half-bracket winner  → champ slot 1
    //   game 2: South vs Midwest half-bracket winner → champ slot 2
    let ff1 = insert_placeholder(conn, 5, None, Some(champ), Some(1))?;
    let ff2 = insert_placeholder(conn, 5, None, Some(champ), Some(2))?;

    // Round 4: Elite Eight (1 per region)
    //   East  → ff1 slot 1 | West  → ff1 slot 2
    //   South → ff2 slot 1 | Midwest → ff2 slot 2
    let e8_east    = insert_placeholder(conn, 4, Some("East"),    Some(ff1), Some(1))?;
    let e8_west    = insert_placeholder(conn, 4, Some("West"),    Some(ff1), Some(2))?;
    let e8_south   = insert_placeholder(conn, 4, Some("South"),   Some(ff2), Some(1))?;
    let e8_midwest = insert_placeholder(conn, 4, Some("Midwest"), Some(ff2), Some(2))?;

    // Round 3: Sweet 16 (2 per region — "A" bracket and "B" bracket within region)
    let s16_east_a    = insert_placeholder(conn, 3, Some("East"),    Some(e8_east),    Some(1))?;
    let s16_east_b    = insert_placeholder(conn, 3, Some("East"),    Some(e8_east),    Some(2))?;
    let s16_west_a    = insert_placeholder(conn, 3, Some("West"),    Some(e8_west),    Some(1))?;
    let s16_west_b    = insert_placeholder(conn, 3, Some("West"),    Some(e8_west),    Some(2))?;
    let s16_south_a   = insert_placeholder(conn, 3, Some("South"),   Some(e8_south),   Some(1))?;
    let s16_south_b   = insert_placeholder(conn, 3, Some("South"),   Some(e8_south),   Some(2))?;
    let s16_midwest_a = insert_placeholder(conn, 3, Some("Midwest"), Some(e8_midwest), Some(1))?;
    let s16_midwest_b = insert_placeholder(conn, 3, Some("Midwest"), Some(e8_midwest), Some(2))?;

    // Round 2: Round of 32 (4 per region)
    // R32 games 0 & 1 feed S16-A; games 2 & 3 feed S16-B.
    // r32_*[i] is the id of the i-th R32 game in that region.
    let r32_east = [
        insert_placeholder(conn, 2, Some("East"), Some(s16_east_a),    Some(1))?,
        insert_placeholder(conn, 2, Some("East"), Some(s16_east_a),    Some(2))?,
        insert_placeholder(conn, 2, Some("East"), Some(s16_east_b),    Some(1))?,
        insert_placeholder(conn, 2, Some("East"), Some(s16_east_b),    Some(2))?,
    ];
    let r32_west = [
        insert_placeholder(conn, 2, Some("West"), Some(s16_west_a),    Some(1))?,
        insert_placeholder(conn, 2, Some("West"), Some(s16_west_a),    Some(2))?,
        insert_placeholder(conn, 2, Some("West"), Some(s16_west_b),    Some(1))?,
        insert_placeholder(conn, 2, Some("West"), Some(s16_west_b),    Some(2))?,
    ];
    let r32_south = [
        insert_placeholder(conn, 2, Some("South"), Some(s16_south_a),  Some(1))?,
        insert_placeholder(conn, 2, Some("South"), Some(s16_south_a),  Some(2))?,
        insert_placeholder(conn, 2, Some("South"), Some(s16_south_b),  Some(1))?,
        insert_placeholder(conn, 2, Some("South"), Some(s16_south_b),  Some(2))?,
    ];
    let r32_midwest = [
        insert_placeholder(conn, 2, Some("Midwest"), Some(s16_midwest_a), Some(1))?,
        insert_placeholder(conn, 2, Some("Midwest"), Some(s16_midwest_a), Some(2))?,
        insert_placeholder(conn, 2, Some("Midwest"), Some(s16_midwest_b), Some(1))?,
        insert_placeholder(conn, 2, Some("Midwest"), Some(s16_midwest_b), Some(2))?,
    ];

    let all_r32_by_region = [r32_east, r32_west, r32_south, r32_midwest];

    // Round 1: Round of 64 (8 per region, actual seeded teams)
    // pairings order: 1v16, 8v9, 5v12, 4v13, 6v11, 3v14, 7v10, 2v15 (indices 0-7)
    // pair_idx 0 → r32[0] slot 1, pair_idx 1 → r32[0] slot 2
    // pair_idx 2 → r32[1] slot 1, pair_idx 3 → r32[1] slot 2
    // pair_idx 4 → r32[2] slot 1, pair_idx 5 → r32[2] slot 2
    // pair_idx 6 → r32[3] slot 1, pair_idx 7 → r32[3] slot 2
    let mut r64_count = 0;
    for (region_idx, region) in regions.iter().enumerate() {
        let region_teams: Vec<_> = teams
            .iter()
            .filter(|(_, r, _, _, _)| r == region)
            .collect();
        let r32_ids = all_r32_by_region[region_idx];

        for (pair_idx, (seed1, seed2)) in pairings.iter().enumerate() {
            let t1_info = region_teams.iter().find(|(s, _, _, _, _)| s == seed1);
            let t2_info = region_teams.iter().find(|(s, _, _, _, _)| s == seed2);

            if let (Some((_, _, name1, _, _)), Some((_, _, name2, _, _))) = (t1_info, t2_info) {
                let team1 = queries::get_team_by_name(conn, name1)?;
                let team2 = queries::get_team_by_name(conn, name2)?;

                if let (Some(t1), Some(t2)) = (team1, team2) {
                    let r32_game_idx = pair_idx / 2;            // which R32 game (0-3)
                    let next_slot = (pair_idx % 2 + 1) as i64; // 1 or 2

                    let m = Matchup {
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
                        next_game_id: Some(r32_ids[r32_game_idx]),
                        next_slot: Some(next_slot),
                    };
                    queries::insert_matchup(conn, &m)?;
                    r64_count += 1;
                }
            }
        }
    }

    println!(
        "Created full bracket graph: {} R64 games + 31 placeholder games (rounds 2-6) \
         with next_game_id links.",
        r64_count
    );
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
