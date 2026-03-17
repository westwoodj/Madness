/// 2026 NCAA Tournament teams — actual bracket released March 15, 2026.
/// Duke is the #1 overall seed (East region).
/// Format: (seed, region, name, abbrev, espn_id)
///
/// ESPN IDs sourced from ESPN's hidden API; a value of "" means unknown.
pub fn teams_2026() -> Vec<(i64, &'static str, &'static str, &'static str, &'static str)> {
    vec![
        // ── EAST ──────────────────────────────────────────────────────────
        (1,  "East",    "Duke",                      "DUKE",  "150"),
        (2,  "East",    "Alabama",                   "ALA",   "333"),
        (3,  "East",    "Wisconsin",                 "WIS",   "275"),
        (4,  "East",    "Arizona",                   "ARIZ",  "12"),
        (5,  "East",    "Oregon",                    "ORE",   "2483"),
        (6,  "East",    "BYU",                       "BYU",   "252"),
        (7,  "East",    "Saint Mary's",              "SMC",   "2608"),
        (8,  "East",    "Mississippi State",         "MSST",  "344"),
        (9,  "East",    "Baylor",                    "BAY",   "239"),
        (10, "East",    "New Mexico",                "UNM",   "167"),
        (11, "East",    "VCU",                       "VCU",   "2670"),
        (12, "East",    "Colorado State",            "CSU",   "36"),
        (13, "East",    "High Point",                "HPU",   "2272"),
        (14, "East",    "Bryant",                    "BRY",   "2825"),
        (15, "East",    "Wofford",                   "WOF",   "2749"),
        (16, "East",    "American",                  "AMER",  "44"),

        // ── WEST ──────────────────────────────────────────────────────────
        (1,  "West",    "Kansas",                    "KAN",   "2305"),
        (2,  "West",    "Michigan State",            "MSU",   "127"),
        (3,  "West",    "Texas A&M",                 "TAMU",  "245"),
        (4,  "West",    "Purdue",                    "PUR",   "2509"),
        (5,  "West",    "Memphis",                   "MEM",   "235"),
        (6,  "West",    "Illinois",                  "ILL",   "356"),
        (7,  "West",    "Clemson",                   "CLEM",  "228"),
        (8,  "West",    "Texas",                     "TEX",   "2641"),
        (9,  "West",    "Utah State",                "USU",   "328"),
        (10, "West",    "Xavier",                    "XAV",   "2752"),
        (11, "West",    "Drake",                     "DRKE",  "2181"),
        (12, "West",    "UC San Diego",              "UCSD",  "2604"),
        (13, "West",    "Akron",                     "AKR",   "2006"),
        (14, "West",    "Montana",                   "MONT",  "149"),
        (15, "West",    "Lipscomb",                  "LIP",   "288"),
        (16, "West",    "SIU Edwardsville",          "SIUE",  "2565"),

        // ── SOUTH ─────────────────────────────────────────────────────────
        (1,  "South",   "Tennessee",                 "TENN",  "2633"),
        (2,  "South",   "Iowa State",                "ISU",   "66"),
        (3,  "South",   "Kentucky",                  "UK",    "96"),
        (4,  "South",   "Florida",                   "FLA",   "57"),
        (5,  "South",   "Missouri",                  "MIZ",   "142"),
        (6,  "South",   "Marquette",                 "MARQ",  "269"),
        (7,  "South",   "Gonzaga",                   "GONZ",  "2250"),
        (8,  "South",   "Georgia",                   "UGA",   "61"),
        (9,  "South",   "Oklahoma",                  "OU",    "201"),
        (10, "South",   "Arkansas",                  "ARK",   "8"),
        (11, "South",   "North Carolina",            "UNC",   "153"),
        (12, "South",   "Liberty",                   "LIB",   "2335"),
        (13, "South",   "Yale",                      "YALE",  "43"),
        (14, "South",   "Vermont",                   "UVM",   "261"),
        (15, "South",   "Rider",                     "RID",   "2520"),
        (16, "South",   "Southern University",       "SOU",   "2582"),

        // ── MIDWEST ───────────────────────────────────────────────────────
        (1,  "Midwest",  "Auburn",                   "AUB",   "2"),
        (2,  "Midwest",  "St. John's",               "SJU",   "2599"),
        (3,  "Midwest",  "UCLA",                     "UCLA",  "26"),
        (4,  "Midwest",  "Maryland",                 "MD",    "120"),
        (5,  "Midwest",  "Creighton",                "CREI",  "156"),
        (6,  "Midwest",  "Mississippi",              "MISS",  "145"),
        (7,  "Midwest",  "Cincinnati",               "CIN",   "2132"),
        (8,  "Midwest",  "Louisville",               "LOU",   "97"),
        (9,  "Midwest",  "Vanderbilt",               "VAN",   "238"),
        (10, "Midwest",  "Nebraska",                 "NEB",   "158"),
        (11, "Midwest",  "UC Irvine",                "UCI",   "2955"),
        (12, "Midwest",  "James Madison",            "JMU",   "256"),
        (13, "Midwest",  "Furman",                   "FUR",   "231"),
        (14, "Midwest",  "South Dakota State",       "SDST",  "2571"),
        (15, "Midwest",  "California Baptist",       "CBU",   "2856"),
        (16, "Midwest",  "Queens",                   "QUEEN", "2858"),
    ]
}

/// Standard seed pairings in the Round of 64 (1v16, 2v15, … 8v9)
/// Returns (higher_seed, lower_seed) pairs
pub fn r64_pairings() -> Vec<(i64, i64)> {
    vec![
        (1, 16),
        (8, 9),
        (5, 12),
        (4, 13),
        (6, 11),
        (3, 14),
        (7, 10),
        (2, 15),
    ]
}
