use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::{models::Prediction, queries};
use crate::model::features::{build_features, FeatureVector};

pub const MODEL_VERSION: &str = "0.1.0-weighted-logistic";

/// Configurable prediction weights loaded from config/model_weights.toml
#[derive(Debug, Clone, Deserialize)]
pub struct ModelWeights {
    pub net_eff_diff: f64,
    pub def_eff_diff: f64,
    pub seed_diff: f64,
    pub sos_diff: f64,
    pub form_diff: f64,
    pub sportsbook_implied: f64,
}

impl Default for ModelWeights {
    fn default() -> Self {
        ModelWeights {
            net_eff_diff: 0.35,
            def_eff_diff: 0.20,
            seed_diff: 0.15,
            sos_diff: 0.10,
            form_diff: 0.10,
            sportsbook_implied: 0.10,
        }
    }
}

impl ModelWeights {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let weights: ModelWeights = toml::from_str(&content)?;
        Ok(weights)
    }

    /// Load from default path, falling back to defaults if not found.
    pub fn load() -> Self {
        ModelWeights::from_file("config/model_weights.toml").unwrap_or_default()
    }
}

/// The result of running the prediction model on a matchup.
#[derive(Debug, Clone, Serialize)]
pub struct PredictionResult {
    pub matchup_id: i64,
    pub team1_id: i64,
    pub team2_id: i64,
    pub team1_name: String,
    pub team2_name: String,
    pub team1_win_prob: f64,
    pub team2_win_prob: f64,
    pub predicted_winner_id: i64,
    pub predicted_winner_name: String,
    pub confidence: f64,
    pub factors: Vec<FactorContribution>,
}

/// Per-factor breakdown for explainability.
#[derive(Debug, Clone, Serialize)]
pub struct FactorContribution {
    pub name: String,
    pub team1_value: f64,
    pub team2_value: f64,
    pub raw_diff: f64,
    pub weighted_contribution: f64,
}

pub fn predict(conn: &Connection, matchup_id: i64, season: i64) -> Result<PredictionResult> {
    let weights = ModelWeights::load();
    let features = build_features(conn, matchup_id, season)?;

    let matchup = queries::get_matchup(conn, matchup_id)?.unwrap();
    let t1_id = matchup.team1_id.unwrap();
    let t2_id = matchup.team2_id.unwrap();
    let t1 = queries::get_team_by_id(conn, t1_id)?.unwrap();
    let t2 = queries::get_team_by_id(conn, t2_id)?.unwrap();

    // Compute logistic regression score
    let score = compute_score(&features, &weights);
    let team1_win_prob = sigmoid(score);
    let team2_win_prob = 1.0 - team1_win_prob;
    let confidence = (team1_win_prob - 0.5).abs() * 2.0;

    let predicted_winner_id = if team1_win_prob >= 0.5 { t1_id } else { t2_id };
    let predicted_winner_name = if team1_win_prob >= 0.5 { t1.name.clone() } else { t2.name.clone() };

    // Build factor breakdown
    let factors = build_factor_breakdown(&features, &weights);

    // Persist prediction
    let factors_json = serde_json::to_string(&factors)?;
    let pred = Prediction {
        id: 0,
        matchup_id,
        team1_win_prob: Some(team1_win_prob),
        team2_win_prob: Some(team2_win_prob),
        predicted_winner_id: Some(predicted_winner_id),
        confidence: Some(confidence),
        model_version: Some(MODEL_VERSION.to_string()),
        factors_json: Some(factors_json),
        created_at: Some(chrono::Utc::now().to_rfc3339()),
    };
    queries::insert_prediction(conn, &pred)?;

    Ok(PredictionResult {
        matchup_id,
        team1_id: t1_id,
        team2_id: t2_id,
        team1_name: t1.name,
        team2_name: t2.name,
        team1_win_prob,
        team2_win_prob,
        predicted_winner_id,
        predicted_winner_name,
        confidence,
        factors,
    })
}

fn compute_score(f: &FeatureVector, w: &ModelWeights) -> f64 {
    // Each feature contributes weight × feature_value to score.
    // Sportsbook implied needs special handling: it's already in probability space [0,1].
    // Convert to logit space: logit(p) = ln(p / (1-p))
    let sb_logit = {
        let p = f.sportsbook_implied.clamp(0.01, 0.99);
        p.ln() - (1.0 - p).ln()
    };

    w.net_eff_diff * f.net_eff_diff
        + w.def_eff_diff * f.def_eff_diff
        + w.seed_diff * f.seed_diff
        + w.sos_diff * f.sos_diff
        + w.form_diff * f.form_diff
        + w.sportsbook_implied * sb_logit
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn build_factor_breakdown(f: &FeatureVector, w: &ModelWeights) -> Vec<FactorContribution> {
    let s1 = &f.team1_stats;
    let s2 = &f.team2_stats;

    let t1_net = s1.as_ref().and_then(|s| s.net_efficiency).unwrap_or(0.0);
    let t2_net = s2.as_ref().and_then(|s| s.net_efficiency).unwrap_or(0.0);
    let t1_def = s1.as_ref().and_then(|s| s.def_efficiency).unwrap_or(100.0);
    let t2_def = s2.as_ref().and_then(|s| s.def_efficiency).unwrap_or(100.0);
    let t1_sos = s1.as_ref().and_then(|s| s.strength_of_schedule).unwrap_or(0.0);
    let t2_sos = s2.as_ref().and_then(|s| s.strength_of_schedule).unwrap_or(0.0);

    vec![
        FactorContribution {
            name: "Net Efficiency".to_string(),
            team1_value: t1_net,
            team2_value: t2_net,
            raw_diff: f.net_eff_diff,
            weighted_contribution: w.net_eff_diff * f.net_eff_diff,
        },
        FactorContribution {
            name: "Defensive Efficiency".to_string(),
            team1_value: t1_def,
            team2_value: t2_def,
            raw_diff: f.def_eff_diff,
            weighted_contribution: w.def_eff_diff * f.def_eff_diff,
        },
        FactorContribution {
            name: "Seed".to_string(),
            team1_value: f.team1_seed as f64,
            team2_value: f.team2_seed as f64,
            raw_diff: f.seed_diff,
            weighted_contribution: w.seed_diff * f.seed_diff,
        },
        FactorContribution {
            name: "Strength of Schedule".to_string(),
            team1_value: t1_sos,
            team2_value: t2_sos,
            raw_diff: f.sos_diff,
            weighted_contribution: w.sos_diff * f.sos_diff,
        },
        FactorContribution {
            name: "Recent Form (L5)".to_string(),
            team1_value: s1.as_ref().and_then(|s| s.last5_wins).unwrap_or(0) as f64,
            team2_value: s2.as_ref().and_then(|s| s.last5_wins).unwrap_or(0) as f64,
            raw_diff: f.form_diff,
            weighted_contribution: w.form_diff * f.form_diff,
        },
        FactorContribution {
            name: "Sportsbook Odds".to_string(),
            team1_value: f.sportsbook_implied,
            team2_value: 1.0 - f.sportsbook_implied,
            raw_diff: f.sportsbook_implied - 0.5,
            weighted_contribution: w.sportsbook_implied * (f.sportsbook_implied - 0.5),
        },
    ]
}
