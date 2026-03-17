pub mod app;
pub mod bracket_view;
pub mod game_view;
pub mod prediction_view;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use rusqlite::Connection;
use std::io;

use app::{App, View};
use crate::db::queries;
use crate::model::predictor;

pub fn run_ui(db_path: &str, season: i64) -> Result<()> {
    let conn = crate::db::open(db_path)?;
    let teams = queries::get_all_teams(&conn)?;
    let matchups = queries::get_all_matchups(&conn)?;

    let mut app = App::new(teams, matchups, season, db_path.to_string());

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app, &conn);

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    conn: &Connection,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Overall layout: title bar | content | status bar
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ])
                .split(size);

            // Title bar
            let title = Paragraph::new(vec![Line::from(vec![
                Span::styled(
                    " 🏀 March Madness 2026 Predictor ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" │ {} ", view_name(&app.view)),
                    Style::default().fg(Color::Cyan),
                ),
            ])])
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Content
            match &app.view {
                View::Bracket => bracket_view::render(f, app, chunks[1]),
                View::GameDetail => game_view::render(f, app, chunks[1]),
                View::Prediction => prediction_view::render(f, app, chunks[1]),
            }

            // Status bar
            let status = Paragraph::new(app.status.as_str())
                .style(Style::default().fg(Color::White));
            f.render_widget(status, chunks[2]);
        })?;

        // Event handling
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Ctrl+C always quits
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                }

                match &app.view {
                    View::Bracket => handle_bracket_keys(app, key.code),
                    View::GameDetail => handle_game_keys(app, conn, key.code)?,
                    View::Prediction => handle_prediction_keys(app, key.code),
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_bracket_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Up | KeyCode::Char('k') => app.navigate_up(),
        KeyCode::Down | KeyCode::Char('j') => app.navigate_down(),
        KeyCode::Enter => {
            if app.selected_matchup().is_some() {
                app.view = View::GameDetail;
                app.status = String::from("p: predict   Esc: back to bracket");
            }
        }
        _ => {}
    }
}

fn handle_game_keys(app: &mut App, conn: &Connection, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Esc => app.go_back(),
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if let Some(matchup) = app.selected_matchup() {
                let matchup_id = matchup.id;
                let season = app.season;
                app.status = String::from("Running prediction...");
                match predictor::predict(conn, matchup_id, season) {
                    Ok(result) => app.set_prediction(result),
                    Err(e) => {
                        app.status = format!("Prediction failed: {} (run `fetch` and `globals` first)", e);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_prediction_keys(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Esc => app.go_back(),
        _ => {}
    }
}

fn view_name(view: &View) -> &'static str {
    match view {
        View::Bracket => "Bracket",
        View::GameDetail => "Game Detail",
        View::Prediction => "Prediction",
    }
}
