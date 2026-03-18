use crate::db::models::{Matchup, Team};
use crate::model::predictor::PredictionResult;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Bracket,
    GameDetail,
    Prediction,
}

pub struct App {
    pub view: View,
    pub teams: Vec<Team>,
    pub matchups: Vec<Matchup>,
    /// Currently selected matchup index in the matchups list
    pub selected_matchup: usize,
    /// Latest prediction result (if any)
    pub prediction: Option<PredictionResult>,
    /// Status message shown in footer
    pub status: String,
    /// Whether we should quit
    pub should_quit: bool,
    /// Current season
    pub season: i64,
    /// DB path
    pub db_path: String,
}

impl App {
    pub fn new(teams: Vec<Team>, matchups: Vec<Matchup>, season: i64, db_path: String) -> Self {
        App {
            view: View::Bracket,
            teams,
            matchups,
            selected_matchup: 0,
            prediction: None,
            status: String::from("↑↓ navigate  Enter select game  p predict  q quit"),
            should_quit: false,
            season,
            db_path,
        }
    }

    pub fn selected_matchup(&self) -> Option<&Matchup> {
        // Only show R64 (round 1) matchups in the bracket view
        let r64: Vec<&Matchup> = self.matchups.iter().filter(|m| m.round == 1).collect();
        r64.get(self.selected_matchup).copied()
    }

    pub fn r64_matchups(&self) -> Vec<&Matchup> {
        self.matchups.iter().filter(|m| m.round == 1).collect()
    }

    pub fn team_by_id(&self, id: i64) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == id)
    }

    pub fn navigate_up(&mut self) {
        let r64_count = self.r64_matchups().len();
        if r64_count == 0 {
            return;
        }
        if self.selected_matchup > 0 {
            self.selected_matchup -= 1;
        } else {
            self.selected_matchup = r64_count - 1;
        }
    }

    pub fn navigate_down(&mut self) {
        let r64_count = self.r64_matchups().len();
        if r64_count == 0 {
            return;
        }
        self.selected_matchup = (self.selected_matchup + 1) % r64_count;
    }

    pub fn set_prediction(&mut self, result: PredictionResult) {
        self.status = format!(
            "Prediction: {} wins ({:.0}% confidence)",
            result.predicted_winner_name,
            result.confidence * 100.0
        );
        self.prediction = Some(result);
        self.view = View::Prediction;
    }

    pub fn go_back(&mut self) {
        match self.view {
            View::Prediction => self.view = View::GameDetail,
            View::GameDetail => {
                self.view = View::Bracket;
                self.status = String::from("↑↓ navigate  Enter select game  p predict  q quit");
            }
            View::Bracket => {}
        }
    }
}
