use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use super::models::{GlobalMetric, Matchup, Prediction, Team, TeamGameStats, TeamSeasonStats};

// ─── Teams ───────────────────────────────────────────────────────────────────

pub fn insert_team(conn: &Connection, team: &Team) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO teams (name, abbrev, seed, region, conference, espn_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            team.name,
            team.abbrev,
            team.seed,
            team.region,
            team.conference,
            team.espn_id
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn upsert_team(conn: &Connection, team: &Team) -> Result<i64> {
    conn.execute(
        "INSERT INTO teams (name, abbrev, seed, region, conference, espn_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(name) DO UPDATE SET
           abbrev = excluded.abbrev,
           seed = excluded.seed,
           region = excluded.region,
           conference = excluded.conference,
           espn_id = excluded.espn_id",
        params![
            team.name,
            team.abbrev,
            team.seed,
            team.region,
            team.conference,
            team.espn_id
        ],
    )?;
    let id = conn.query_row(
        "SELECT id FROM teams WHERE name = ?1",
        params![team.name],
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn get_all_teams(conn: &Connection) -> Result<Vec<Team>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, abbrev, seed, region, conference, espn_id FROM teams ORDER BY region, seed",
    )?;
    let teams = stmt
        .query_map([], |row| {
            Ok(Team {
                id: row.get(0)?,
                name: row.get(1)?,
                abbrev: row.get(2)?,
                seed: row.get(3)?,
                region: row.get(4)?,
                conference: row.get(5)?,
                espn_id: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to query teams")?;
    Ok(teams)
}

pub fn get_team_by_id(conn: &Connection, id: i64) -> Result<Option<Team>> {
    let result = conn.query_row(
        "SELECT id, name, abbrev, seed, region, conference, espn_id FROM teams WHERE id = ?1",
        params![id],
        |row| {
            Ok(Team {
                id: row.get(0)?,
                name: row.get(1)?,
                abbrev: row.get(2)?,
                seed: row.get(3)?,
                region: row.get(4)?,
                conference: row.get(5)?,
                espn_id: row.get(6)?,
            })
        },
    );
    match result {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_team_by_name(conn: &Connection, name: &str) -> Result<Option<Team>> {
    let result = conn.query_row(
        "SELECT id, name, abbrev, seed, region, conference, espn_id FROM teams WHERE name = ?1",
        params![name],
        |row| {
            Ok(Team {
                id: row.get(0)?,
                name: row.get(1)?,
                abbrev: row.get(2)?,
                seed: row.get(3)?,
                region: row.get(4)?,
                conference: row.get(5)?,
                espn_id: row.get(6)?,
            })
        },
    );
    match result {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

// ─── Season Stats ─────────────────────────────────────────────────────────────

pub fn upsert_season_stats(conn: &Connection, s: &TeamSeasonStats) -> Result<()> {
    conn.execute(
        "INSERT INTO team_season_stats (
            team_id, season, wins, losses,
            points_per_game, points_allowed_per_game,
            field_goal_pct, three_point_pct, free_throw_pct,
            rebounds_per_game, off_rebounds_per_game, def_rebounds_per_game,
            assists_per_game, turnovers_per_game, steals_per_game, blocks_per_game,
            pace, off_efficiency, def_efficiency, net_efficiency,
            net_ranking, strength_of_schedule, last5_wins, tournament_odds_ml
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24
        )
        ON CONFLICT(team_id, season) DO UPDATE SET
            wins = excluded.wins, losses = excluded.losses,
            points_per_game = excluded.points_per_game,
            points_allowed_per_game = excluded.points_allowed_per_game,
            field_goal_pct = excluded.field_goal_pct,
            three_point_pct = excluded.three_point_pct,
            free_throw_pct = excluded.free_throw_pct,
            rebounds_per_game = excluded.rebounds_per_game,
            off_rebounds_per_game = excluded.off_rebounds_per_game,
            def_rebounds_per_game = excluded.def_rebounds_per_game,
            assists_per_game = excluded.assists_per_game,
            turnovers_per_game = excluded.turnovers_per_game,
            steals_per_game = excluded.steals_per_game,
            blocks_per_game = excluded.blocks_per_game,
            pace = excluded.pace,
            off_efficiency = excluded.off_efficiency,
            def_efficiency = excluded.def_efficiency,
            net_efficiency = excluded.net_efficiency,
            net_ranking = excluded.net_ranking,
            strength_of_schedule = excluded.strength_of_schedule,
            last5_wins = excluded.last5_wins,
            tournament_odds_ml = excluded.tournament_odds_ml",
        params![
            s.team_id, s.season, s.wins, s.losses,
            s.points_per_game, s.points_allowed_per_game,
            s.field_goal_pct, s.three_point_pct, s.free_throw_pct,
            s.rebounds_per_game, s.off_rebounds_per_game, s.def_rebounds_per_game,
            s.assists_per_game, s.turnovers_per_game, s.steals_per_game, s.blocks_per_game,
            s.pace, s.off_efficiency, s.def_efficiency, s.net_efficiency,
            s.net_ranking, s.strength_of_schedule, s.last5_wins, s.tournament_odds_ml
        ],
    )?;
    Ok(())
}

pub fn get_season_stats(
    conn: &Connection,
    team_id: i64,
    season: i64,
) -> Result<Option<TeamSeasonStats>> {
    let result = conn.query_row(
        "SELECT id, team_id, season, wins, losses,
                points_per_game, points_allowed_per_game,
                field_goal_pct, three_point_pct, free_throw_pct,
                rebounds_per_game, off_rebounds_per_game, def_rebounds_per_game,
                assists_per_game, turnovers_per_game, steals_per_game, blocks_per_game,
                pace, off_efficiency, def_efficiency, net_efficiency,
                net_ranking, strength_of_schedule, last5_wins, tournament_odds_ml
         FROM team_season_stats WHERE team_id = ?1 AND season = ?2",
        params![team_id, season],
        row_to_season_stats,
    );
    match result {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_all_season_stats(conn: &Connection, season: i64) -> Result<Vec<TeamSeasonStats>> {
    let mut stmt = conn.prepare(
        "SELECT id, team_id, season, wins, losses,
                points_per_game, points_allowed_per_game,
                field_goal_pct, three_point_pct, free_throw_pct,
                rebounds_per_game, off_rebounds_per_game, def_rebounds_per_game,
                assists_per_game, turnovers_per_game, steals_per_game, blocks_per_game,
                pace, off_efficiency, def_efficiency, net_efficiency,
                net_ranking, strength_of_schedule, last5_wins, tournament_odds_ml
         FROM team_season_stats WHERE season = ?1",
    )?;
    let stats = stmt
        .query_map(params![season], row_to_season_stats)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to query season stats")?;
    Ok(stats)
}

fn row_to_season_stats(row: &rusqlite::Row) -> rusqlite::Result<TeamSeasonStats> {
    Ok(TeamSeasonStats {
        id: row.get(0)?,
        team_id: row.get(1)?,
        season: row.get(2)?,
        wins: row.get(3)?,
        losses: row.get(4)?,
        points_per_game: row.get(5)?,
        points_allowed_per_game: row.get(6)?,
        field_goal_pct: row.get(7)?,
        three_point_pct: row.get(8)?,
        free_throw_pct: row.get(9)?,
        rebounds_per_game: row.get(10)?,
        off_rebounds_per_game: row.get(11)?,
        def_rebounds_per_game: row.get(12)?,
        assists_per_game: row.get(13)?,
        turnovers_per_game: row.get(14)?,
        steals_per_game: row.get(15)?,
        blocks_per_game: row.get(16)?,
        pace: row.get(17)?,
        off_efficiency: row.get(18)?,
        def_efficiency: row.get(19)?,
        net_efficiency: row.get(20)?,
        net_ranking: row.get(21)?,
        strength_of_schedule: row.get(22)?,
        last5_wins: row.get(23)?,
        tournament_odds_ml: row.get(24)?,
    })
}

// ─── Game Stats ──────────────────────────────────────────────────────────────

pub fn insert_game_stats(conn: &Connection, g: &TeamGameStats) -> Result<i64> {
    conn.execute(
        "INSERT INTO team_game_stats (
            team_id, opponent_id, game_date, season, is_home, is_tournament,
            points, points_allowed, fg_pct, three_pct, ft_pct,
            rebounds, off_rebounds, def_rebounds, assists, turnovers, steals, blocks, won
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        params![
            g.team_id, g.opponent_id, g.game_date, g.season, g.is_home, g.is_tournament,
            g.points, g.points_allowed, g.fg_pct, g.three_pct, g.ft_pct,
            g.rebounds, g.off_rebounds, g.def_rebounds, g.assists, g.turnovers, g.steals, g.blocks, g.won
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_recent_games(
    conn: &Connection,
    team_id: i64,
    season: i64,
    limit: i64,
) -> Result<Vec<TeamGameStats>> {
    let mut stmt = conn.prepare(
        "SELECT id, team_id, opponent_id, game_date, season, is_home, is_tournament,
                points, points_allowed, fg_pct, three_pct, ft_pct,
                rebounds, off_rebounds, def_rebounds, assists, turnovers, steals, blocks, won
         FROM team_game_stats WHERE team_id = ?1 AND season = ?2
         ORDER BY game_date DESC LIMIT ?3",
    )?;
    let games = stmt
        .query_map(params![team_id, season, limit], |row| {
            Ok(TeamGameStats {
                id: row.get(0)?,
                team_id: row.get(1)?,
                opponent_id: row.get(2)?,
                game_date: row.get(3)?,
                season: row.get(4)?,
                is_home: row.get(5)?,
                is_tournament: row.get(6)?,
                points: row.get(7)?,
                points_allowed: row.get(8)?,
                fg_pct: row.get(9)?,
                three_pct: row.get(10)?,
                ft_pct: row.get(11)?,
                rebounds: row.get(12)?,
                off_rebounds: row.get(13)?,
                def_rebounds: row.get(14)?,
                assists: row.get(15)?,
                turnovers: row.get(16)?,
                steals: row.get(17)?,
                blocks: row.get(18)?,
                won: row.get(19)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to query game stats")?;
    Ok(games)
}

// ─── Global Metrics ───────────────────────────────────────────────────────────

pub fn upsert_global_metric(conn: &Connection, m: &GlobalMetric) -> Result<()> {
    conn.execute(
        "INSERT INTO global_metrics (season, metric_name, avg_value, std_dev, min_value, max_value)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(season, metric_name) DO UPDATE SET
           avg_value = excluded.avg_value,
           std_dev = excluded.std_dev,
           min_value = excluded.min_value,
           max_value = excluded.max_value",
        params![m.season, m.metric_name, m.avg_value, m.std_dev, m.min_value, m.max_value],
    )?;
    Ok(())
}

pub fn get_global_metric(
    conn: &Connection,
    season: i64,
    metric_name: &str,
) -> Result<Option<GlobalMetric>> {
    let result = conn.query_row(
        "SELECT id, season, metric_name, avg_value, std_dev, min_value, max_value
         FROM global_metrics WHERE season = ?1 AND metric_name = ?2",
        params![season, metric_name],
        |row| {
            Ok(GlobalMetric {
                id: row.get(0)?,
                season: row.get(1)?,
                metric_name: row.get(2)?,
                avg_value: row.get(3)?,
                std_dev: row.get(4)?,
                min_value: row.get(5)?,
                max_value: row.get(6)?,
            })
        },
    );
    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_all_global_metrics(conn: &Connection, season: i64) -> Result<Vec<GlobalMetric>> {
    let mut stmt = conn.prepare(
        "SELECT id, season, metric_name, avg_value, std_dev, min_value, max_value
         FROM global_metrics WHERE season = ?1 ORDER BY metric_name",
    )?;
    let metrics = stmt
        .query_map(params![season], |row| {
            Ok(GlobalMetric {
                id: row.get(0)?,
                season: row.get(1)?,
                metric_name: row.get(2)?,
                avg_value: row.get(3)?,
                std_dev: row.get(4)?,
                min_value: row.get(5)?,
                max_value: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to query global metrics")?;
    Ok(metrics)
}

// ─── Matchups ────────────────────────────────────────────────────────────────

pub fn insert_matchup(conn: &Connection, m: &Matchup) -> Result<i64> {
    conn.execute(
        "INSERT INTO matchups (round, region, team1_id, team2_id, team1_ml, team2_ml,
          spread, over_under, winner_id, game_date)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![
            m.round, m.region, m.team1_id, m.team2_id,
            m.team1_ml, m.team2_ml, m.spread, m.over_under,
            m.winner_id, m.game_date
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_matchup_odds(
    conn: &Connection,
    matchup_id: i64,
    team1_ml: Option<i64>,
    team2_ml: Option<i64>,
    spread: Option<f64>,
    over_under: Option<f64>,
) -> Result<()> {
    conn.execute(
        "UPDATE matchups SET team1_ml=?2, team2_ml=?3, spread=?4, over_under=?5 WHERE id=?1",
        params![matchup_id, team1_ml, team2_ml, spread, over_under],
    )?;
    Ok(())
}

pub fn update_matchup_winner(conn: &Connection, matchup_id: i64, winner_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE matchups SET winner_id=?2 WHERE id=?1",
        params![matchup_id, winner_id],
    )?;
    Ok(())
}

pub fn get_matchup(conn: &Connection, id: i64) -> Result<Option<Matchup>> {
    let result = conn.query_row(
        "SELECT id, round, region, team1_id, team2_id, team1_ml, team2_ml,
                spread, over_under, winner_id, game_date
         FROM matchups WHERE id = ?1",
        params![id],
        row_to_matchup,
    );
    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_matchups_by_round(conn: &Connection, round: i64) -> Result<Vec<Matchup>> {
    let mut stmt = conn.prepare(
        "SELECT id, round, region, team1_id, team2_id, team1_ml, team2_ml,
                spread, over_under, winner_id, game_date
         FROM matchups WHERE round = ?1 ORDER BY region, id",
    )?;
    let matchups = stmt
        .query_map(params![round], row_to_matchup)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to query matchups")?;
    Ok(matchups)
}

pub fn get_all_matchups(conn: &Connection) -> Result<Vec<Matchup>> {
    let mut stmt = conn.prepare(
        "SELECT id, round, region, team1_id, team2_id, team1_ml, team2_ml,
                spread, over_under, winner_id, game_date
         FROM matchups ORDER BY round, region, id",
    )?;
    let matchups = stmt
        .query_map([], row_to_matchup)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("Failed to query matchups")?;
    Ok(matchups)
}

fn row_to_matchup(row: &rusqlite::Row) -> rusqlite::Result<Matchup> {
    Ok(Matchup {
        id: row.get(0)?,
        round: row.get(1)?,
        region: row.get(2)?,
        team1_id: row.get(3)?,
        team2_id: row.get(4)?,
        team1_ml: row.get(5)?,
        team2_ml: row.get(6)?,
        spread: row.get(7)?,
        over_under: row.get(8)?,
        winner_id: row.get(9)?,
        game_date: row.get(10)?,
    })
}

// ─── Predictions ─────────────────────────────────────────────────────────────

pub fn insert_prediction(conn: &Connection, p: &Prediction) -> Result<i64> {
    conn.execute(
        "INSERT INTO predictions (matchup_id, team1_win_prob, team2_win_prob,
          predicted_winner_id, confidence, model_version, factors_json, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            p.matchup_id, p.team1_win_prob, p.team2_win_prob,
            p.predicted_winner_id, p.confidence, p.model_version,
            p.factors_json, p.created_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_latest_prediction(conn: &Connection, matchup_id: i64) -> Result<Option<Prediction>> {
    let result = conn.query_row(
        "SELECT id, matchup_id, team1_win_prob, team2_win_prob, predicted_winner_id,
                confidence, model_version, factors_json, created_at
         FROM predictions WHERE matchup_id = ?1 ORDER BY id DESC LIMIT 1",
        params![matchup_id],
        |row| {
            Ok(Prediction {
                id: row.get(0)?,
                matchup_id: row.get(1)?,
                team1_win_prob: row.get(2)?,
                team2_win_prob: row.get(3)?,
                predicted_winner_id: row.get(4)?,
                confidence: row.get(5)?,
                model_version: row.get(6)?,
                factors_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        },
    );
    match result {
        Ok(p) => Ok(Some(p)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
