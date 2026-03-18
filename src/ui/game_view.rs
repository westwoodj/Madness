use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::db::models::Team;
use crate::ui::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let matchup = match app.selected_matchup() {
        Some(m) => m,
        None => {
            let p = Paragraph::new("No matchup selected. Press Esc to go back.")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(p, area);
            return;
        }
    };

    let t1 = matchup.team1_id.and_then(|id| app.team_by_id(id));
    let t2 = matchup.team2_id.and_then(|id| app.team_by_id(id));

    // Layout: header | stats table | odds row | footer hint
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // header
            Constraint::Min(18),    // stats table
            Constraint::Length(4),  // odds
            Constraint::Length(2),  // footer
        ])
        .split(area);

    // ── Header ───────────────────────────────────────────────────────────────
    let header_text = format!(
        "{} ({}) vs {} ({})",
        t1.map(|t| t.name.as_str()).unwrap_or("TBD"),
        t1.and_then(|t| t.seed).map(|s| format!("#{}", s)).unwrap_or_default(),
        t2.map(|t| t.name.as_str()).unwrap_or("TBD"),
        t2.and_then(|t| t.seed).map(|s| format!("#{}", s)).unwrap_or_default(),
    );
    let region = matchup.region.as_deref().unwrap_or("?");
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            header_text,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Region: {}  |  Round of 64", region),
            Style::default().fg(Color::Cyan),
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Game Detail "));
    f.render_widget(header, chunks[0]);

    // We don't have stats in the TUI directly — we read from app which
    // doesn't carry TeamSeasonStats yet. We display placeholders / seed info.
    let t1_info = TeamDisplayInfo::from_team(t1);
    let t2_info = TeamDisplayInfo::from_team(t2);

    // ── Stats Table ──────────────────────────────────────────────────────────
    let stat_rows: Vec<Row> = vec![
        stat_row("Seed", &t1_info.seed, &t2_info.seed),
        stat_row("Conference", &t1_info.conference, &t2_info.conference),
        stat_row("Region", &t1_info.region, &t2_info.region),
    ];

    let header_row = Row::new(vec![
        Cell::from("Stat").style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
        Cell::from(t1_info.name.clone()).style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
        Cell::from(t2_info.name.clone()).style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
    ]);

    let table = Table::new(
        stat_rows,
        [Constraint::Percentage(35), Constraint::Percentage(32), Constraint::Percentage(33)],
    )
    .header(header_row)
    .block(Block::default().borders(Borders::ALL).title(" Team Stats "))
    .column_spacing(1);
    f.render_widget(table, chunks[1]);

    // ── Odds ─────────────────────────────────────────────────────────────────
    let ml1 = matchup.team1_ml.map(|v| format!("{:+}", v)).unwrap_or_else(|| "N/A".to_string());
    let ml2 = matchup.team2_ml.map(|v| format!("{:+}", v)).unwrap_or_else(|| "N/A".to_string());
    let spread = matchup.spread.map(|v| format!("{:+.1}", v)).unwrap_or_else(|| "N/A".to_string());
    let ou = matchup.over_under.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "N/A".to_string());

    let odds_text = format!(
        "Moneyline: {} {} vs {} {}    Spread: {}    O/U: {}",
        t1_info.name, ml1, t2_info.name, ml2, spread, ou
    );
    let odds = Paragraph::new(odds_text)
        .style(Style::default().fg(Color::Magenta))
        .block(Block::default().borders(Borders::ALL).title(" Sportsbook Odds "));
    f.render_widget(odds, chunks[2]);

    // ── Footer ───────────────────────────────────────────────────────────────
    let footer = Paragraph::new("p: run prediction   Esc: back to bracket")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[3]);
}

fn stat_row<'a>(label: &'a str, v1: &'a str, v2: &'a str) -> Row<'a> {
    Row::new(vec![
        Cell::from(label).style(Style::default().fg(Color::White)),
        Cell::from(v1).style(Style::default().fg(Color::Green)),
        Cell::from(v2).style(Style::default().fg(Color::Red)),
    ])
}

struct TeamDisplayInfo {
    name: String,
    seed: String,
    conference: String,
    region: String,
}

impl TeamDisplayInfo {
    fn from_team(t: Option<&Team>) -> Self {
        match t {
            Some(team) => TeamDisplayInfo {
                name: team.name.clone(),
                seed: team.seed.map(|s| s.to_string()).unwrap_or_else(|| "?".to_string()),
                conference: team.conference.clone().unwrap_or_else(|| "?".to_string()),
                region: team.region.clone().unwrap_or_else(|| "?".to_string()),
            },
            None => TeamDisplayInfo {
                name: "TBD".to_string(),
                seed: "?".to_string(),
                conference: "?".to_string(),
                region: "?".to_string(),
            },
        }
    }
}
