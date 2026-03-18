mod db;
mod fetch;
mod model;
mod tournament;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};

const DB_PATH: &str = "madness.db";
const SEASON: i64 = 2026;

#[derive(Parser)]
#[command(name = "madness", about = "March Madness 2026 predictor", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// SQLite database path
    #[arg(long, default_value = DB_PATH)]
    db: String,

    /// Season year
    #[arg(long, default_value_t = SEASON)]
    season: i64,
}

#[derive(Subcommand)]
enum Commands {
    /// Seed the DB with the 2026 tournament bracket (68 teams + R64 matchups)
    Bracket,
    /// Fetch team stats and odds from ESPN API
    Fetch,
    /// Compute global metrics (averages) across all tournament teams
    Globals,
    /// Run the prediction model for a matchup and print results
    Predict {
        /// Matchup ID (from the matchups table)
        matchup_id: i64,
    },
    /// Launch the interactive TUI bracket viewer
    Ui,
    /// Print all matchups with IDs (for use with `predict`)
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = &cli.db;
    let season = cli.season;

    let conn = db::open(db_path)?;
    db::init(&conn)?;

    match cli.command.unwrap_or(Commands::Ui) {
        Commands::Bracket => {
            println!("Seeding 2026 bracket...");
            tournament::bracket::seed_bracket(&conn)?;
        }
        Commands::Fetch => {
            println!("Fetching stats from ESPN (season {})...", season);
            fetch::fetch_all_stats(&conn, season)?;
        }
        Commands::Globals => {
            println!("Computing global metrics for season {}...", season);
            model::features::compute_global_metrics(&conn, season)?;
        }
        Commands::Predict { matchup_id } => {
            let result = model::predictor::predict(&conn, matchup_id, season)?;
            println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  {} vs {}", result.team1_name, result.team2_name);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Predicted winner: {}", result.predicted_winner_name);
            println!(
                "  {} win prob:  {:.1}%",
                result.team1_name,
                result.team1_win_prob * 100.0
            );
            println!(
                "  {} win prob: {:.1}%",
                result.team2_name,
                result.team2_win_prob * 100.0
            );
            println!("  Confidence: {:.0}%", result.confidence * 100.0);
            println!("\n  Factor breakdown:");
            for f in &result.factors {
                println!(
                    "    {:25} {:>8.2} vs {:>8.2}   contrib: {:+.3}",
                    f.name, f.team1_value, f.team2_value, f.weighted_contribution
                );
            }
        }
        Commands::Ui => {
            ui::run_ui(db_path, season)?;
        }
        Commands::List => {
            let matchups = db::queries::get_all_matchups(&conn)?;
            if matchups.is_empty() {
                println!("No matchups found. Run `madness bracket` first.");
            } else {
                println!("{:<5} {:<12} {:<10} {}", "ID", "Round", "Region", "Matchup");
                println!("{}", "─".repeat(60));
                for m in &matchups {
                    let t1 = m.team1_id
                        .and_then(|id| db::queries::get_team_by_id(&conn, id).ok().flatten())
                        .map(|t| t.name)
                        .unwrap_or_else(|| "TBD".to_string());
                    let t2 = m.team2_id
                        .and_then(|id| db::queries::get_team_by_id(&conn, id).ok().flatten())
                        .map(|t| t.name)
                        .unwrap_or_else(|| "TBD".to_string());
                    println!(
                        "{:<5} {:<12} {:<10} {} vs {}",
                        m.id,
                        db::models::round_name(m.round),
                        m.region.as_deref().unwrap_or("?"),
                        t1,
                        t2
                    );
                }
            }
        }
    }

    Ok(())
}
