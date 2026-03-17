use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::ui::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let regions = ["East", "West", "South", "Midwest"];
    let r64: Vec<_> = app.r64_matchups();

    // Split into 4 region columns
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    for (col_idx, region) in regions.iter().enumerate() {
        let region_matchups: Vec<(usize, &&crate::db::models::Matchup)> = r64
            .iter()
            .enumerate()
            .filter(|(_, m)| m.region.as_deref() == Some(region))
            .collect();

        let items: Vec<ListItem> = region_matchups
            .iter()
            .map(|(global_idx, matchup)| {
                let t1_name = matchup
                    .team1_id
                    .and_then(|id| app.team_by_id(id))
                    .map(|t| format!("({}) {}", t.seed.unwrap_or(0), t.name.as_str()))
                    .unwrap_or_else(|| "TBD".to_string());
                let t2_name = matchup
                    .team2_id
                    .and_then(|id| app.team_by_id(id))
                    .map(|t| format!("({}) {}", t.seed.unwrap_or(0), t.name.as_str()))
                    .unwrap_or_else(|| "TBD".to_string());

                let is_selected = *global_idx == app.selected_matchup;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                // Truncate to fit column
                let max_len = 20;
                let t1 = truncate(&t1_name, max_len);
                let t2 = truncate(&t2_name, max_len);

                let winner_marker = |tid: Option<i64>| -> &str {
                    if matchup.winner_id.is_some() && matchup.winner_id == tid {
                        " ✓"
                    } else {
                        ""
                    }
                };

                let line1 = format!("{}{}", t1, winner_marker(matchup.team1_id));
                let line2 = format!("  vs");
                let line3 = format!("{}{}", t2, winner_marker(matchup.team2_id));
                let divider = "─".repeat(22);

                ListItem::new(vec![
                    Line::from(Span::styled(line1, style)),
                    Line::from(Span::styled(line2, Style::default().fg(Color::DarkGray))),
                    Line::from(Span::styled(line3, style)),
                    Line::from(Span::styled(divider, Style::default().fg(Color::DarkGray))),
                ])
            })
            .collect();

        let mut list_state = ListState::default();
        // Set selected state only for the relevant region column
        let local_selected = region_matchups
            .iter()
            .position(|(gi, _)| *gi == app.selected_matchup);
        list_state.select(local_selected);

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", region))
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        f.render_stateful_widget(list, cols[col_idx], &mut list_state);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
