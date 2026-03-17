pub const CREATE_TEAMS: &str = "
CREATE TABLE IF NOT EXISTS teams (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    abbrev      TEXT,
    seed        INTEGER,
    region      TEXT,
    conference  TEXT,
    espn_id     TEXT
);";

pub const CREATE_TEAM_SEASON_STATS: &str = "
CREATE TABLE IF NOT EXISTS team_season_stats (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id                 INTEGER NOT NULL REFERENCES teams(id),
    season                  INTEGER NOT NULL,
    wins                    INTEGER,
    losses                  INTEGER,
    points_per_game         REAL,
    points_allowed_per_game REAL,
    field_goal_pct          REAL,
    three_point_pct         REAL,
    free_throw_pct          REAL,
    rebounds_per_game       REAL,
    off_rebounds_per_game   REAL,
    def_rebounds_per_game   REAL,
    assists_per_game        REAL,
    turnovers_per_game      REAL,
    steals_per_game         REAL,
    blocks_per_game         REAL,
    pace                    REAL,
    off_efficiency          REAL,
    def_efficiency          REAL,
    net_efficiency          REAL,
    net_ranking             INTEGER,
    strength_of_schedule    REAL,
    last5_wins              INTEGER,
    tournament_odds_ml      INTEGER,
    UNIQUE(team_id, season)
);";

pub const CREATE_TEAM_GAME_STATS: &str = "
CREATE TABLE IF NOT EXISTS team_game_stats (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    team_id         INTEGER NOT NULL REFERENCES teams(id),
    opponent_id     INTEGER REFERENCES teams(id),
    game_date       TEXT,
    season          INTEGER NOT NULL,
    is_home         INTEGER,
    is_tournament   INTEGER,
    points          INTEGER,
    points_allowed  INTEGER,
    fg_pct          REAL,
    three_pct       REAL,
    ft_pct          REAL,
    rebounds        INTEGER,
    off_rebounds    INTEGER,
    def_rebounds    INTEGER,
    assists         INTEGER,
    turnovers       INTEGER,
    steals          INTEGER,
    blocks          INTEGER,
    won             INTEGER
);";

pub const CREATE_GLOBAL_METRICS: &str = "
CREATE TABLE IF NOT EXISTS global_metrics (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    season       INTEGER NOT NULL,
    metric_name  TEXT NOT NULL,
    avg_value    REAL,
    std_dev      REAL,
    min_value    REAL,
    max_value    REAL,
    UNIQUE(season, metric_name)
);";

pub const CREATE_MATCHUPS: &str = "
CREATE TABLE IF NOT EXISTS matchups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    round       INTEGER NOT NULL,
    region      TEXT,
    team1_id    INTEGER REFERENCES teams(id),
    team2_id    INTEGER REFERENCES teams(id),
    team1_ml    INTEGER,
    team2_ml    INTEGER,
    spread      REAL,
    over_under  REAL,
    winner_id   INTEGER REFERENCES teams(id),
    game_date   TEXT
);";

pub const CREATE_PREDICTIONS: &str = "
CREATE TABLE IF NOT EXISTS predictions (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    matchup_id          INTEGER NOT NULL REFERENCES matchups(id),
    team1_win_prob      REAL,
    team2_win_prob      REAL,
    predicted_winner_id INTEGER REFERENCES teams(id),
    confidence          REAL,
    model_version       TEXT,
    factors_json        TEXT,
    created_at          TEXT
);";
