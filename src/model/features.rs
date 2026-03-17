use anyhow::{bail, Result};
use rusqlite::Connection;

use crate::db::{models::TeamSeasonStats, queries};
use crate::fetch::odds::ml_to_implied_prob;

/// All features used by the prediction model, normalized to [0,1] or as differentials.
#[derive(Debug, Clone)]
pub struct FeatureVector {
    /// Net efficiency margin differential (team1 - team2), z-score normalized
    pub net_eff_diff: f64,
    /// Defensive efficiency differential (team2_def - team1_def; higher = team1 better defense)
    pub def_eff_diff: f64,
    /// Seed differential (team2.seed - team1.seed; positive = team1 is lower seed / stronger)
    pub seed_diff: f64,
    /// Strength of schedule differential (team1 - team2)
    pub sos_diff: f64,
    /// Last-5-game win rate differential (team1 - team2) in range [-1, 1]
    pub form_diff: f64,
    /// Sportsbook implied probability for team1 (0.5 if no odds)
    pub sportsbook_implied: f64,
    /// Raw stats for display purposes
    pub team1_stats: Option<TeamSeasonStats>,
    pub team2_stats: Option<TeamSeasonStats>,
    pub team1_seed: i64,
    pub team2_seed: i64,
}

pub fn build_features(
    conn: &Connection,
    matchup_id: i64,
    season: i64,
) -> Result<FeatureVector> {
    let matchup = queries::get_matchup(conn, matchup_id)?
        .ok_or_else(|| anyhow::anyhow!("Matchup {} not found", matchup_id))?;

    let t1_id = matchup.team1_id.ok_or_else(|| anyhow::anyhow!("Matchup has no team1"))?;
    let t2_id = matchup.team2_id.ok_or_else(|| anyhow::anyhow!("Matchup has no team2"))?;

    let t1 = queries::get_team_by_id(conn, t1_id)?
        .ok_or_else(|| anyhow::anyhow!("Team {} not found", t1_id))?;
    let t2 = queries::get_team_by_id(conn, t2_id)?
        .ok_or_else(|| anyhow::anyhow!("Team {} not found", t2_id))?;

    let s1 = queries::get_season_stats(conn, t1_id, season)?;
    let s2 = queries::get_season_stats(conn, t2_id, season)?;

    // ── Global metric normalization helpers ────────────────────────────────
    let global_std = |metric: &str| -> f64 {
        queries::get_global_metric(conn, season, metric)
            .ok()
            .flatten()
            .and_then(|m| m.std_dev)
            .unwrap_or(1.0)
            .max(0.001) // avoid divide-by-zero
    };

    // ── Net efficiency differential ────────────────────────────────────────
    let net_eff_diff = {
        let v1 = s1.as_ref().and_then(|s| s.net_efficiency).unwrap_or(0.0);
        let v2 = s2.as_ref().and_then(|s| s.net_efficiency).unwrap_or(0.0);
        let std = global_std("net_efficiency");
        (v1 - v2) / std
    };

    // ── Defensive efficiency differential ─────────────────────────────────
    // Lower def_efficiency = better defense; so team1 advantage = team2_def - team1_def
    let def_eff_diff = {
        let v1 = s1.as_ref().and_then(|s| s.def_efficiency).unwrap_or(100.0);
        let v2 = s2.as_ref().and_then(|s| s.def_efficiency).unwrap_or(100.0);
        let std = global_std("def_efficiency");
        (v2 - v1) / std
    };

    // ── Seed differential ─────────────────────────────────────────────────
    let team1_seed = t1.seed.unwrap_or(8);
    let team2_seed = t2.seed.unwrap_or(8);
    // Higher seed number = weaker team; team1 advantage = team2.seed - team1.seed
    let seed_diff = (team2_seed - team1_seed) as f64 / 15.0; // normalize to [-1,1]

    // ── Strength of schedule differential ────────────────────────────────
    let sos_diff = {
        let v1 = s1.as_ref().and_then(|s| s.strength_of_schedule).unwrap_or(0.0);
        let v2 = s2.as_ref().and_then(|s| s.strength_of_schedule).unwrap_or(0.0);
        let std = global_std("strength_of_schedule");
        (v1 - v2) / std
    };

    // ── Recent form differential ──────────────────────────────────────────
    let form_rate = |stats: &Option<TeamSeasonStats>| -> f64 {
        stats.as_ref()
            .and_then(|s| s.last5_wins)
            .map(|w| w as f64 / 5.0)
            .unwrap_or(0.5)
    };
    let form_diff = form_rate(&s1) - form_rate(&s2);

    // ── Sportsbook implied probability ────────────────────────────────────
    let sportsbook_implied = match (matchup.team1_ml, matchup.team2_ml) {
        (Some(ml1), Some(ml2)) => {
            let p1 = ml_to_implied_prob(ml1);
            let p2 = ml_to_implied_prob(ml2);
            // Normalize to remove bookmaker vig
            p1 / (p1 + p2)
        }
        (Some(ml1), None) => ml_to_implied_prob(ml1),
        _ => 0.5, // no odds available
    };

    Ok(FeatureVector {
        net_eff_diff,
        def_eff_diff,
        seed_diff,
        sos_diff,
        form_diff,
        sportsbook_implied,
        team1_stats: s1,
        team2_stats: s2,
        team1_seed,
        team2_seed,
    })
}

/// Compute and store global metrics (mean, std dev) for all tournament teams.
pub fn compute_global_metrics(conn: &Connection, season: i64) -> Result<()> {
    let all_stats = queries::get_all_season_stats(conn, season)?;
    if all_stats.is_empty() {
        bail!("No season stats found for season {}. Run `fetch` first.", season);
    }

    let metrics: Vec<(&str, Box<dyn Fn(&crate::db::models::TeamSeasonStats) -> Option<f64>>)> = vec![
        ("points_per_game",         Box::new(|s| s.points_per_game)),
        ("points_allowed_per_game", Box::new(|s| s.points_allowed_per_game)),
        ("field_goal_pct",          Box::new(|s| s.field_goal_pct)),
        ("three_point_pct",         Box::new(|s| s.three_point_pct)),
        ("free_throw_pct",          Box::new(|s| s.free_throw_pct)),
        ("rebounds_per_game",       Box::new(|s| s.rebounds_per_game)),
        ("assists_per_game",        Box::new(|s| s.assists_per_game)),
        ("turnovers_per_game",      Box::new(|s| s.turnovers_per_game)),
        ("pace",                    Box::new(|s| s.pace)),
        ("off_efficiency",          Box::new(|s| s.off_efficiency)),
        ("def_efficiency",          Box::new(|s| s.def_efficiency)),
        ("net_efficiency",          Box::new(|s| s.net_efficiency)),
        ("strength_of_schedule",    Box::new(|s| s.strength_of_schedule)),
    ];

    for (name, extractor) in &metrics {
        let values: Vec<f64> = all_stats
            .iter()
            .filter_map(|s| extractor(s))
            .collect();

        if values.is_empty() {
            continue;
        }

        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let metric = crate::db::models::GlobalMetric {
            id: 0,
            season,
            metric_name: name.to_string(),
            avg_value: Some(avg),
            std_dev: Some(std_dev),
            min_value: Some(min),
            max_value: Some(max),
        };
        queries::upsert_global_metric(conn, &metric)?;
        println!("  {}: avg={:.2}, std={:.2}, min={:.2}, max={:.2}", name, avg, std_dev, min, max);
    }

    println!("Global metrics computed for {} teams.", all_stats.len());
    Ok(())
}
