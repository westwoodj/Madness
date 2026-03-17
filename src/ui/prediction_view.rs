use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::ui::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let result = match &app.prediction {
        Some(r) => r,
        None => {
            let p = Paragraph::new("No prediction available. Press 'p' in the game view.")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(p, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // header
            Constraint::Length(5),  // probability bars
            Constraint::Length(3),  // confidence
            Constraint::Min(10),    // factor table
            Constraint::Length(2),  // footer
        ])
        .split(area);

    // ── Header ───────────────────────────────────────────────────────────────
    let winner_style = Style::default().fg(Color::Green).add_modifier(Modifier::BOLD);
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            format!("Prediction: {} wins", result.predicted_winner_name),
            winner_style,
        )),
        Line::from(Span::styled(
            format!(
                "{} vs {}  |  Model v{}",
                result.team1_name,
                result.team2_name,
                crate::model::predictor::MODEL_VERSION
            ),
            Style::default().fg(Color::Cyan),
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Prediction Result "));
    f.render_widget(header, chunks[0]);

    // ── Probability bars ─────────────────────────────────────────────────────
    let prob_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(chunks[1]);

    let gauge1 = Gauge::default()
        .block(Block::default().title(format!(
            " {} ",
            result.team1_name
        )))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(result.team1_win_prob)
        .label(format!("{:.1}%", result.team1_win_prob * 100.0));
    f.render_widget(gauge1, prob_chunks[0]);

    let gauge2 = Gauge::default()
        .block(Block::default().title(format!(
            " {} ",
            result.team2_name
        )))
        .gauge_style(Style::default().fg(Color::Red))
        .ratio(result.team2_win_prob)
        .label(format!("{:.1}%", result.team2_win_prob * 100.0));
    f.render_widget(gauge2, prob_chunks[1]);

    // ── Confidence meter ─────────────────────────────────────────────────────
    let conf_color = if result.confidence > 0.7 {
        Color::Green
    } else if result.confidence > 0.4 {
        Color::Yellow
    } else {
        Color::Red
    };
    let conf_gauge = Gauge::default()
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT).title(" Confidence "))
        .gauge_style(Style::default().fg(conf_color))
        .ratio(result.confidence)
        .label(format!("{:.0}%", result.confidence * 100.0));
    f.render_widget(conf_gauge, chunks[2]);

    // ── Factor breakdown table ────────────────────────────────────────────────
    let header_row = Row::new(vec![
        Cell::from("Factor").style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
        Cell::from(result.team1_name.as_str()).style(Style::default().fg(Color::Green)),
        Cell::from(result.team2_name.as_str()).style(Style::default().fg(Color::Red)),
        Cell::from("Contribution").style(Style::default().fg(Color::Yellow)),
    ]);

    let factor_rows: Vec<Row> = result
        .factors
        .iter()
        .map(|f| {
            let contrib_style = if f.weighted_contribution > 0.01 {
                Style::default().fg(Color::Green)
            } else if f.weighted_contribution < -0.01 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                Cell::from(f.name.as_str()),
                Cell::from(format!("{:.2}", f.team1_value)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.2}", f.team2_value)).style(Style::default().fg(Color::Red)),
                Cell::from(format!("{:+.3}", f.weighted_contribution)).style(contrib_style),
            ])
        })
        .collect();

    let table = Table::new(
        factor_rows,
        [
            Constraint::Percentage(32),
            Constraint::Percentage(22),
            Constraint::Percentage(22),
            Constraint::Percentage(24),
        ],
    )
    .header(header_row)
    .block(Block::default().borders(Borders::ALL).title(" Factor Breakdown (positive = favors team 1) "))
    .column_spacing(1);
    f.render_widget(table, chunks[3]);

    // ── Footer ───────────────────────────────────────────────────────────────
    let footer = Paragraph::new("Esc: back to game   q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[4]);
}
