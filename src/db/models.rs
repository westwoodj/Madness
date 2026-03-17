use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub abbrev: Option<String>,
    pub seed: Option<i64>,
    pub region: Option<String>,
    pub conference: Option<String>,
    pub espn_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSeasonStats {
    pub id: i64,
    pub team_id: i64,
    pub season: i64,
    pub wins: Option<i64>,
    pub losses: Option<i64>,
    pub points_per_game: Option<f64>,
    pub points_allowed_per_game: Option<f64>,
    pub field_goal_pct: Option<f64>,
    pub three_point_pct: Option<f64>,
    pub free_throw_pct: Option<f64>,
    pub rebounds_per_game: Option<f64>,
    pub off_rebounds_per_game: Option<f64>,
    pub def_rebounds_per_game: Option<f64>,
    pub assists_per_game: Option<f64>,
    pub turnovers_per_game: Option<f64>,
    pub steals_per_game: Option<f64>,
    pub blocks_per_game: Option<f64>,
    pub pace: Option<f64>,
    pub off_efficiency: Option<f64>,
    pub def_efficiency: Option<f64>,
    pub net_efficiency: Option<f64>,
    pub net_ranking: Option<i64>,
    pub strength_of_schedule: Option<f64>,
    pub last5_wins: Option<i64>,
    pub tournament_odds_ml: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamGameStats {
    pub id: i64,
    pub team_id: i64,
    pub opponent_id: Option<i64>,
    pub game_date: Option<String>,
    pub season: i64,
    pub is_home: Option<i64>,
    pub is_tournament: Option<i64>,
    pub points: Option<i64>,
    pub points_allowed: Option<i64>,
    pub fg_pct: Option<f64>,
    pub three_pct: Option<f64>,
    pub ft_pct: Option<f64>,
    pub rebounds: Option<i64>,
    pub off_rebounds: Option<i64>,
    pub def_rebounds: Option<i64>,
    pub assists: Option<i64>,
    pub turnovers: Option<i64>,
    pub steals: Option<i64>,
    pub blocks: Option<i64>,
    pub won: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetric {
    pub id: i64,
    pub season: i64,
    pub metric_name: String,
    pub avg_value: Option<f64>,
    pub std_dev: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matchup {
    pub id: i64,
    pub round: i64,
    pub region: Option<String>,
    pub team1_id: Option<i64>,
    pub team2_id: Option<i64>,
    pub team1_ml: Option<i64>,
    pub team2_ml: Option<i64>,
    pub spread: Option<f64>,
    pub over_under: Option<f64>,
    pub winner_id: Option<i64>,
    pub game_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub id: i64,
    pub matchup_id: i64,
    pub team1_win_prob: Option<f64>,
    pub team2_win_prob: Option<f64>,
    pub predicted_winner_id: Option<i64>,
    pub confidence: Option<f64>,
    pub model_version: Option<String>,
    pub factors_json: Option<String>,
    pub created_at: Option<String>,
}

/// Round labels for display
pub fn round_name(round: i64) -> &'static str {
    match round {
        0 => "First Four",
        1 => "Round of 64",
        2 => "Round of 32",
        3 => "Sweet 16",
        4 => "Elite Eight",
        5 => "Final Four",
        6 => "Championship",
        _ => "Unknown",
    }
}
