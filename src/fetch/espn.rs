use anyhow::{Context, Result};
use reqwest::blocking::Client;
use rusqlite::Connection;
use serde_json::Value;

use crate::db::{models::TeamSeasonStats, queries};

const BASE: &str = "https://site.api.espn.com/apis/site/v2/sports/basketball/mens-college-basketball";

pub struct EspnClient {
    client: Client,
}

impl EspnClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; madness/0.1)")
            .timeout(std::time::Duration::from_secs(15))
            .build()?;
        Ok(EspnClient { client })
    }

    /// Fetch team statistics from ESPN and store in DB.
    pub fn fetch_team_stats(&self, conn: &Connection, team_id: i64, espn_id: &str, season: i64) -> Result<()> {
        let url = format!("{BASE}/teams/{espn_id}/statistics?season={season}");
        let resp: Value = self
            .client
            .get(&url)
            .send()
            .context("ESPN stats request failed")?
            .json()
            .context("ESPN stats JSON parse failed")?;

        let stats = parse_team_statistics(&resp, team_id, season)?;
        queries::upsert_season_stats(conn, &stats)?;
        Ok(())
    }

    /// Fetch the current tournament scoreboard (live odds and scores).
    pub fn fetch_scoreboard(&self, conn: &Connection, date: &str) -> Result<()> {
        let url = format!("{BASE}/scoreboard?dates={date}&groups=100");
        let resp: Value = self
            .client
            .get(&url)
            .send()
            .context("ESPN scoreboard request failed")?
            .json()
            .context("ESPN scoreboard JSON parse failed")?;

        parse_scoreboard_odds(conn, &resp)?;
        Ok(())
    }

    /// Fetch per-game schedule/results for a team.
    pub fn fetch_team_schedule(&self, conn: &Connection, team_id: i64, espn_id: &str, season: i64) -> Result<usize> {
        let url = format!("{BASE}/teams/{espn_id}/schedule?season={season}");
        let resp: Value = self
            .client
            .get(&url)
            .send()
            .context("ESPN schedule request failed")?
            .json()
            .context("ESPN schedule JSON parse failed")?;

        let count = parse_schedule_games(conn, &resp, team_id, season)?;
        Ok(count)
    }
}

// ─── Parsers ──────────────────────────────────────────────────────────────────

fn parse_team_statistics(json: &Value, team_id: i64, season: i64) -> Result<TeamSeasonStats> {
    // ESPN stats response: json["team"]["statistics"]["splits"]["categories"]
    let cats = json
        .pointer("/team/statistics/splits/categories")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut s = TeamSeasonStats {
        id: 0,
        team_id,
        season,
        wins: None,
        losses: None,
        points_per_game: None,
        points_allowed_per_game: None,
        field_goal_pct: None,
        three_point_pct: None,
        free_throw_pct: None,
        rebounds_per_game: None,
        off_rebounds_per_game: None,
        def_rebounds_per_game: None,
        assists_per_game: None,
        turnovers_per_game: None,
        steals_per_game: None,
        blocks_per_game: None,
        pace: None,
        off_efficiency: None,
        def_efficiency: None,
        net_efficiency: None,
        net_ranking: None,
        strength_of_schedule: None,
        last5_wins: None,
        tournament_odds_ml: None,
    };

    for cat in &cats {
        let cat_name = cat["name"].as_str().unwrap_or("");
        let stats = cat["stats"].as_array().cloned().unwrap_or_default();

        for stat in &stats {
            let name = stat["name"].as_str().unwrap_or("");
            let val: Option<f64> = stat["value"].as_f64();

            match (cat_name, name) {
                (_, "avgPoints") | (_, "pointsPerGame") => s.points_per_game = val,
                (_, "avgPointsAllowed") => s.points_allowed_per_game = val,
                (_, "fieldGoalPct") => s.field_goal_pct = val,
                (_, "threePointPct") | (_, "avgThreePointPct") => s.three_point_pct = val,
                (_, "freeThrowPct") => s.free_throw_pct = val,
                (_, "avgRebounds") | (_, "reboundsPerGame") => s.rebounds_per_game = val,
                (_, "avgOffRebounds") => s.off_rebounds_per_game = val,
                (_, "avgDefRebounds") => s.def_rebounds_per_game = val,
                (_, "avgAssists") | (_, "assistsPerGame") => s.assists_per_game = val,
                (_, "avgTurnovers") | (_, "turnoversPerGame") => s.turnovers_per_game = val,
                (_, "avgSteals") => s.steals_per_game = val,
                (_, "avgBlocks") => s.blocks_per_game = val,
                (_, "wins") => s.wins = val.map(|v| v as i64),
                (_, "losses") => s.losses = val.map(|v| v as i64),
                _ => {}
            }
        }
    }

    // Also try top-level record
    if let Some(record) = json.pointer("/team/record/items/0") {
        if let Some(summary) = record["summary"].as_str() {
            // e.g. "29-5"
            let parts: Vec<&str> = summary.split('-').collect();
            if parts.len() == 2 {
                s.wins = parts[0].parse().ok();
                s.losses = parts[1].parse().ok();
            }
        }
    }

    Ok(s)
}

fn parse_scoreboard_odds(conn: &Connection, json: &Value) -> Result<()> {
    let events = json["events"].as_array().cloned().unwrap_or_default();

    for event in &events {
        let competitions = event["competitions"].as_array().cloned().unwrap_or_default();
        for comp in &competitions {
            // Try to get odds
            let odds = comp.pointer("/odds/0");
            if odds.is_none() {
                continue;
            }
            let odds = odds.unwrap();

            let spread: Option<f64> = odds["spread"].as_f64();
            let over_under: Option<f64> = odds["overUnder"].as_f64();

            // Get competitors
            let competitors = comp["competitors"].as_array().cloned().unwrap_or_default();
            if competitors.len() < 2 {
                continue;
            }

            let team1_espn_id = competitors[0].pointer("/team/id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let team2_espn_id = competitors[1].pointer("/team/id").and_then(|v| v.as_str()).unwrap_or("").to_string();

            // Find the matchup in DB by team espn IDs
            let t1 = conn.query_row(
                "SELECT id FROM teams WHERE espn_id = ?1",
                rusqlite::params![team1_espn_id],
                |r| r.get::<_, i64>(0),
            ).ok();
            let t2 = conn.query_row(
                "SELECT id FROM teams WHERE espn_id = ?1",
                rusqlite::params![team2_espn_id],
                |r| r.get::<_, i64>(0),
            ).ok();

            if let (Some(t1_id), Some(t2_id)) = (t1, t2) {
                let matchup_id: Option<i64> = conn.query_row(
                    "SELECT id FROM matchups WHERE
                     (team1_id = ?1 AND team2_id = ?2) OR (team1_id = ?2 AND team2_id = ?1)",
                    rusqlite::params![t1_id, t2_id],
                    |r| r.get(0),
                ).ok();

                if let Some(mid) = matchup_id {
                    // Parse moneylines from odds
                    let team1_ml = odds.pointer("/homeTeamOdds/moneyLine").and_then(|v| v.as_i64());
                    let team2_ml = odds.pointer("/awayTeamOdds/moneyLine").and_then(|v| v.as_i64());

                    queries::update_matchup_odds(conn, mid, team1_ml, team2_ml, spread, over_under)?;
                }
            }
        }
    }
    Ok(())
}

fn parse_schedule_games(
    conn: &Connection,
    json: &Value,
    team_id: i64,
    season: i64,
) -> Result<usize> {
    let events = json["events"].as_array().cloned().unwrap_or_default();
    let mut count = 0;

    for event in &events {
        let game_date = event["date"].as_str().map(|s| s[..10].to_string());
        let competitions = event["competitions"].as_array().cloned().unwrap_or_default();

        for comp in &competitions {
            let competitors = comp["competitors"].as_array().cloned().unwrap_or_default();
            if competitors.len() < 2 {
                continue;
            }

            // Find which competitor is our team
            let (my_comp, opp_comp, is_home) = {
                let _home_id = competitors[0].pointer("/team/id").and_then(|v| v.as_str()).unwrap_or("");
                if competitors[0].pointer("/team/id").and_then(|v| v.as_str())
                    .map(|id| {
                        conn.query_row("SELECT id FROM teams WHERE espn_id = ?1", rusqlite::params![id], |r| r.get::<_, i64>(0)).ok()
                    })
                    .flatten() == Some(team_id)
                {
                    (&competitors[0], &competitors[1], true)
                } else {
                    (&competitors[1], &competitors[0], false)
                }
            };

            let my_score = my_comp.pointer("/score/value").and_then(|v| v.as_f64()).map(|v| v as i64);
            let opp_score = opp_comp.pointer("/score/value").and_then(|v| v.as_f64()).map(|v| v as i64);
            let won = match (my_score, opp_score) {
                (Some(m), Some(o)) => Some(if m > o { 1i64 } else { 0i64 }),
                _ => None,
            };

            let opp_espn_id = opp_comp.pointer("/team/id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let opponent_id: Option<i64> = conn.query_row(
                "SELECT id FROM teams WHERE espn_id = ?1",
                rusqlite::params![opp_espn_id],
                |r| r.get(0),
            ).ok();

            // Try to get box score stats from competition
            let stats = my_comp["statistics"].as_array().cloned().unwrap_or_default();
            let stat = |name: &str| -> Option<f64> {
                stats.iter()
                    .find(|s| s["name"].as_str() == Some(name))
                    .and_then(|s| s["displayValue"].as_str())
                    .and_then(|v| v.replace('%', "").parse::<f64>().ok())
            };

            let game = crate::db::models::TeamGameStats {
                id: 0,
                team_id,
                opponent_id,
                game_date: game_date.clone(),
                season,
                is_home: Some(if is_home { 1 } else { 0 }),
                is_tournament: Some(0),
                points: my_score,
                points_allowed: opp_score,
                fg_pct: stat("fieldGoalPct").or_else(|| stat("FG%")),
                three_pct: stat("threePointPct").or_else(|| stat("3P%")),
                ft_pct: stat("freeThrowPct").or_else(|| stat("FT%")),
                rebounds: stat("totalRebounds").map(|v| v as i64),
                off_rebounds: stat("offensiveRebounds").map(|v| v as i64),
                def_rebounds: stat("defensiveRebounds").map(|v| v as i64),
                assists: stat("assists").map(|v| v as i64),
                turnovers: stat("turnovers").map(|v| v as i64),
                steals: stat("steals").map(|v| v as i64),
                blocks: stat("blocks").map(|v| v as i64),
                won,
            };

            queries::insert_game_stats(conn, &game)?;
            count += 1;
        }
    }

    Ok(count)
}
