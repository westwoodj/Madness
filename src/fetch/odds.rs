/// Convert an American moneyline to implied win probability.
/// e.g. +150 → 0.40, -200 → 0.667
pub fn ml_to_implied_prob(ml: i64) -> f64 {
    if ml > 0 {
        100.0 / (ml as f64 + 100.0)
    } else {
        (-ml as f64) / (-ml as f64 + 100.0)
    }
}

/// Parse a moneyline string like "+150" or "-200" into an integer.
pub fn parse_moneyline(s: &str) -> Option<i64> {
    let s = s.trim().replace(' ', "");
    s.parse::<i64>().ok()
}
