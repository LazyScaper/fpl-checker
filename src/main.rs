use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const PLAYER_AND_TEAM_IDS: [FplTeamInfo; 8] = [
    FplTeamInfo {
        owner: "Dan",
        team_id: 396409,
    },
    FplTeamInfo {
        owner: "Jake",
        team_id: 2239760,
    },
    FplTeamInfo {
        owner: "Jay",
        team_id: 2186577,
    },
    FplTeamInfo {
        owner: "Shane",
        team_id: 258293,
    },
    FplTeamInfo {
        owner: "Dylan",
        team_id: 761504,
    },
    FplTeamInfo {
        owner: "Harry",
        team_id: 7718758,
    },
    FplTeamInfo {
        owner: "Josh",
        team_id: 2242306,
    },
    FplTeamInfo {
        owner: "Ed",
        team_id: 8828197,
    },
];

const NEWLY_PROMOTED_CLUBS: [i64; 3] = [3, 11, 17];
const BOOTSTRAP_DATA_URI: &'static str = "https://fantasy.premierleague.com/api/bootstrap-static/";

#[derive(Deserialize)]
struct BootstrapTeam {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct BootstrapElement {
    id: i64,
    web_name: String,
    now_cost: f64,
    team: i64,
}

#[derive(Deserialize)]
struct BootstrapData {
    elements: Vec<BootstrapElement>,
    teams: Vec<BootstrapTeam>,
}

#[derive(Deserialize)]
struct PicksData {
    picks: Vec<PickElement>,
}

#[derive(Deserialize)]
struct PickElement {
    is_captain: bool,
    element: i64,
}

#[derive(Deserialize)]
struct GameweekData {
    current_event: i64,
    name: String,
}

#[derive(Debug, PartialEq, Clone, Default)]
struct ValidationResult {
    is_valid: bool,
    reason: String,
}

impl ValidationResult {
    fn valid() -> Self {
        Self {
            is_valid: true,
            reason: "".to_string(),
        }
    }

    fn invalid(reason: &str) -> Self {
        Self {
            is_valid: false,
            reason: reason.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
struct Club {
    id: i64,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
struct Player {
    id: i64,
    name: String,
    price_in_millions: f64,
    club: Club,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Team {
    id: i64,
    name: String,
    owner: String,
    captain: Player,
    players: Vec<Player>,
}

struct FplTeamInfo {
    owner: &'static str,
    team_id: i64,
}

fn build_clubs_by_id(bootstrap_data: &BootstrapData) -> HashMap<i64, String> {
    let mut clubs_by_id: HashMap<i64, String> = HashMap::new();

    for club in &bootstrap_data.teams {
        clubs_by_id.insert(club.id, club.name.clone());
    }

    clubs_by_id
}

fn build_players_by_id(
    clubs_by_club_id: &HashMap<i64, String>,
    bootstrap_data: &BootstrapData,
) -> HashMap<i64, Player> {
    let mut players_by_id: HashMap<i64, Player> = HashMap::new();

    for element in &bootstrap_data.elements {
        let player = Player {
            id: element.id,
            name: element.web_name.clone(),
            price_in_millions: element.now_cost / 10.0,
            club: Club {
                id: element.team,
                name: match { clubs_by_club_id.get(&element.team) } {
                    Some(team_name) => team_name.to_string(),
                    _ => {
                        panic!("Could not find a team")
                    }
                },
            },
        };

        players_by_id.insert(element.id, player);
    }

    players_by_id
}

fn build_team(
    team_id: i64,
    owner: &str,
    players_by_player_id: &HashMap<i64, Player>,
    picks_data: &PicksData,
    gameweek_data: &GameweekData,
) -> Team {
    let mut players = Vec::new();
    let mut captain = Player::default();

    for pick in &picks_data.picks {
        let id = pick.element;
        let player = Player {
            id,
            name: players_by_player_id.get(&id).unwrap().name.clone(),
            price_in_millions: players_by_player_id.get(&id).unwrap().price_in_millions,
            club: Club {
                id: players_by_player_id.get(&id).unwrap().club.id,
                name: players_by_player_id.get(&id).unwrap().club.name.to_string(),
            },
        };

        if pick.is_captain {
            captain = player.clone();
        }

        players.push(player);
    }

    Team {
        id: team_id,
        name: gameweek_data.name.clone().to_string(),
        owner: owner.to_string(),
        captain,
        players,
    }
}

fn team_contains_players_under_10_m(team: &Team) -> ValidationResult {
    let mut players_above_price_threshold: IndexMap<String, f64> = IndexMap::new();

    for player in &team.players {
        if player.price_in_millions >= 10.0 {
            players_above_price_threshold.insert(player.name.clone(), player.price_in_millions);
        }
    }

    let mut violation_string: String = format!(
        "Big wompers! {} has gone overbudget with ",
        team.owner.clone()
    );

    for (index, (player_name, price)) in players_above_price_threshold.iter().enumerate() {
        if index == 0 {
        } else if index == players_above_price_threshold.len() - 1 {
            violation_string.push_str(" and ");
        } else {
            violation_string.push_str(", ");
        }
        violation_string.push_str(&format!("{} ({}m)", &player_name, price));
    }

    if players_above_price_threshold.len() > 0 {
        return ValidationResult::invalid(&violation_string);
    }

    ValidationResult::valid()
}

fn team_contains_at_most_one_player_per_club(team: &Team) -> ValidationResult {
    let mut seen_players_by_club_id: HashMap<i64, Player> = HashMap::new();

    for player in &team.players {
        if seen_players_by_club_id.contains_key(&player.club.id) {
            return ValidationResult::invalid(&format!(
                "{} has shat the bed. {} contains more than 1 player from {} ({} and {})",
                &team.owner,
                &team.name,
                &player.club.name,
                seen_players_by_club_id.get(&player.club.id).unwrap().name,
                &player.name
            ));
        };
        seen_players_by_club_id.insert(player.club.id, player.clone());
    }

    ValidationResult::valid()
}

fn team_contains_players_from_newly_promoted_clubs(
    clubs_by_club_id: &HashMap<i64, String>,
    team: &Team,
) -> ValidationResult {
    for club_id in NEWLY_PROMOTED_CLUBS {
        if !team.players.iter().any(|player| player.club.id == club_id) {
            return ValidationResult::invalid(&format!(
                "Yikes! {} has not included players from {}. That's gonna sting",
                team.owner,
                clubs_by_club_id.get(&club_id).unwrap()
            ));
        }
    }

    ValidationResult::valid()
}

fn fetch_data_as_json<T>(uri: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let data = ureq::get(uri).call()?.into_body().read_json::<T>()?;
    Ok(data)
}

fn main() {
    let bootstrap_data: BootstrapData = fetch_data_as_json(BOOTSTRAP_DATA_URI)
        .expect("Something went wrong fetching bootstrap data");

    let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
    let players_by_id = build_players_by_id(&clubs_by_club_id, &bootstrap_data);

    let mut rules_breakers: Vec<ValidationResult> = Vec::new();

    for fpl_team in PLAYER_AND_TEAM_IDS {
        let gameweek_data: GameweekData = fetch_data_as_json(&format!(
            "https://fantasy.premierleague.com/api/entry/{}/",
            fpl_team.team_id
        ))
        .expect("Something went wrong fetching gameweek data");

        let picks_data: PicksData = fetch_data_as_json(&format!(
            "https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/",
            fpl_team.team_id, gameweek_data.current_event
        ))
        .expect("Something went wrong fetching picks data");

        let team = build_team(
            fpl_team.team_id,
            fpl_team.owner,
            &players_by_id,
            &picks_data,
            &gameweek_data,
        );

        let result = team_contains_players_under_10_m(&team);

        if !result.is_valid {
            rules_breakers.push(result);
        }

        let result = team_contains_players_from_newly_promoted_clubs(&clubs_by_club_id, &team);

        if !result.is_valid {
            rules_breakers.push(result);
        }

        let result = team_contains_at_most_one_player_per_club(&team);

        if !result.is_valid {
            rules_breakers.push(result);
        }
    }

    if rules_breakers.is_empty() {
        println!("No rules breakers found");
    } else {
        for rule_broken in rules_breakers {
            println!("{}\n\n", rule_broken.reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    const BOOTSTRAP_JSON: &str = include_str!("../tests/samples/bootstrap.json");
    const GAMEWEEK_JSON: &str = include_str!("../tests/samples/gameweek.json");
    const PICKS_JSON: &str = include_str!("../tests/samples/picks.json");
    const VALID_TEAM_JSON: &str = include_str!("../tests/samples/valid_team.json");
    const INVALID_TEAM_JSON: &str = include_str!("../tests/samples/invalid_team.json");

    #[test]
    fn should_build_clubs_by_club_id_from_bootstrap_data() {
        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let actual = build_clubs_by_id(&bootstrap_data);

        assert_eq!(Some(&"Arsenal".to_string()), actual.get(&1));
        assert_eq!(Some(&"Burnley".to_string()), actual.get(&3));
        assert_eq!(Some(&"Brighton".to_string()), actual.get(&6));
        assert_eq!(Some(&"Man City".to_string()), actual.get(&13));
    }

    #[test]
    fn should_build_players_by_id_from_bootstrap_data() {
        let partial_expected = Player {
            id: 249,
            name: "João Pedro".to_string(),
            price_in_millions: 7.5,
            club: Club {
                id: 7,
                name: "Chelsea".to_string(),
            },
        };

        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let actual = build_players_by_id(&clubs_by_club_id, &bootstrap_data);

        assert_eq!(Some(&partial_expected), actual.get(&partial_expected.id));
    }

    #[test]
    fn should_build_team_from_data() {
        let expected = Team {
            id: 2239760,
            name: "Pedro Cask Ale".to_string(),
            owner: "Jake".to_string(),
            captain: Player {
                id: 249,
                name: "João Pedro".to_string(),
                price_in_millions: 7.5,
                club: Club {
                    id: 7,
                    name: "Chelsea".to_string(),
                },
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Pickford".to_string(),
                    price_in_millions: 5.5,
                    club: Club {
                        id: 9,

                        name: "Everton".to_string(),
                    },
                },
                Player {
                    id: 145,
                    name: "De Cuyper".to_string(),
                    price_in_millions: 4.5,
                    club: Club {
                        id: 6,

                        name: "Brighton".to_string(),
                    },
                },
                Player {
                    id: 506,
                    name: "Murillo".to_string(),
                    price_in_millions: 5.5,
                    club: Club {
                        id: 16,

                        name: "Nott'm Forest".to_string(),
                    },
                },
                Player {
                    id: 348,
                    name: "Rodon".to_string(),
                    price_in_millions: 4.0,
                    club: Club {
                        id: 11,

                        name: "Leeds".to_string(),
                    },
                },
                Player {
                    id: 119,
                    name: "Mbeumo".to_string(),
                    price_in_millions: 8.0,
                    club: Club {
                        id: 14,

                        name: "Man Utd".to_string(),
                    },
                },
                Player {
                    id: 382,
                    name: "Wirtz".to_string(),
                    price_in_millions: 8.5,
                    club: Club {
                        id: 12,

                        name: "Liverpool".to_string(),
                    },
                },
                Player {
                    id: 413,
                    name: "Marmoush".to_string(),
                    price_in_millions: 8.5,
                    club: Club {
                        id: 13,

                        name: "Man City".to_string(),
                    },
                },
                Player {
                    id: 582,
                    name: "Kudus".to_string(),
                    price_in_millions: 6.5,
                    club: Club {
                        id: 18,

                        name: "Spurs".to_string(),
                    },
                },
                Player {
                    id: 666,
                    name: "Gyökeres".to_string(),
                    price_in_millions: 9.0,
                    club: Club {
                        id: 1,

                        name: "Arsenal".to_string(),
                    },
                },
                Player {
                    id: 249,
                    name: "João Pedro".to_string(),
                    price_in_millions: 7.5,
                    club: Club {
                        id: 7,

                        name: "Chelsea".to_string(),
                    },
                },
                Player {
                    id: 624,
                    name: "Bowen".to_string(),
                    price_in_millions: 8.0,
                    club: Club {
                        id: 19,

                        name: "West Ham".to_string(),
                    },
                },
                Player {
                    id: 470,
                    name: "Dúbravka".to_string(),
                    price_in_millions: 4.0,
                    club: Club {
                        id: 3,

                        name: "Burnley".to_string(),
                    },
                },
                Player {
                    id: 486,
                    name: "Elanga".to_string(),
                    price_in_millions: 7.0,
                    club: Club {
                        id: 15,

                        name: "Newcastle".to_string(),
                    },
                },
                Player {
                    id: 541,
                    name: "Reinildo".to_string(),
                    price_in_millions: 4.0,
                    club: Club {
                        id: 17,

                        name: "Sunderland".to_string(),
                    },
                },
                Player {
                    id: 256,
                    name: "Muñoz".to_string(),
                    price_in_millions: 5.5,
                    club: Club {
                        id: 8,

                        name: "Crystal Palace".to_string(),
                    },
                },
            ],
        };

        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let gameweek_data: GameweekData =
            from_str(&GAMEWEEK_JSON).expect("Something went wrong parsing gameweek data");
        let picks_data: PicksData =
            from_str(&PICKS_JSON).expect("Something went wrong parsing picks data");

        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let players_by_player_id = build_players_by_id(&clubs_by_club_id.clone(), &bootstrap_data);
        let actual = build_team(
            2239760,
            "Jake",
            &players_by_player_id,
            &picks_data,
            &gameweek_data,
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn should_fail_if_team_has_more_than_one_player_from_a_club() {
        let team = Team {
            id: 2239760,
            name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 287,
                name: "Pickford".to_string(),
                price_in_millions: 7.5,
                club: Club {
                    id: 9,

                    name: "Everton".to_string(),
                },
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Pickford".to_string(),
                    price_in_millions: 7.5,
                    club: Club {
                        id: 9,

                        name: "Everton".to_string(),
                    },
                },
                Player {
                    id: 291,
                    name: "James Tarkowski".to_string(),
                    price_in_millions: 5.5,
                    club: Club {
                        id: 9,

                        name: "Everton".to_string(),
                    },
                },
            ],
        };
        let actual = team_contains_at_most_one_player_per_club(&team);
        let expected = ValidationResult::invalid(
            "Jake Peters has shat the bed. Pedro Cask Ale contains more than 1 player from Everton (Pickford and James Tarkowski)",
        );

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_pass_if_team_does_not_have_more_than_one_player_from_a_club() {
        let team: Team =
            from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let actual = team_contains_at_most_one_player_per_club(&team);
        let expected = ValidationResult::valid();

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_fail_if_team_has_player_above_price_limit() {
        let team = Team {
            id: 2239760,
            name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 287,
                name: "Pickford".to_string(),
                price_in_millions: 7.5,
                club: Club {
                    id: 9,

                    name: "Everton".to_string(),
                },
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Pickford".to_string(),
                    price_in_millions: 7.5,
                    club: Club {
                        id: 9,

                        name: "Everton".to_string(),
                    },
                },
                Player {
                    id: 291,
                    name: "James Tarkowski".to_string(),
                    price_in_millions: 10.5,
                    club: Club {
                        id: 9,

                        name: "Everton".to_string(),
                    },
                },
            ],
        };
        let actual = team_contains_players_under_10_m(&team);
        let expected = ValidationResult::invalid(
            "Big wompers! Jake Peters has gone overbudget with James Tarkowski (10.5m)",
        );

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_produce_multiple_failures_if_team_has_more_than_1_player_above_price_limit() {
        let team = from_str(&INVALID_TEAM_JSON).expect("Something went wrong parsing invalid team");
        let actual = team_contains_players_under_10_m(&team);
        let expected = ValidationResult::invalid(
            "Big wompers! Javier Rufo has gone overbudget with Palmer (10.5m) and Haaland (14m)",
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_pass_if_team_has_players_under_price_limit() {
        let team: Team =
            from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let actual = team_contains_players_under_10_m(&team);
        let expected = ValidationResult::valid();

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_fail_if_team_does_not_have_players_from_newly_promoted_clubs() {
        let team = Team {
            id: 2239760,
            name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 287,
                name: "Pickford".to_string(),
                price_in_millions: 7.5,
                club: Club {
                    id: 9,

                    name: "Everton".to_string(),
                },
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Pickford".to_string(),
                    price_in_millions: 7.5,
                    club: Club {
                        id: 9,

                        name: "Everton".to_string(),
                    },
                },
                Player {
                    id: 291,
                    name: "James Tarkowski".to_string(),
                    price_in_millions: 10.5,
                    club: Club {
                        id: 9,

                        name: "Everton".to_string(),
                    },
                },
                Player {
                    id: 348,
                    name: "Rodon".to_string(),
                    price_in_millions: 4.0,
                    club: Club {
                        id: 11,

                        name: "Leeds".to_string(),
                    },
                },
                Player {
                    id: 541,
                    name: "Reinildo".to_string(),
                    price_in_millions: 4.0,
                    club: Club {
                        id: 17,

                        name: "Sunderland".to_string(),
                    },
                },
            ],
        };

        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let actual = team_contains_players_from_newly_promoted_clubs(&clubs_by_club_id, &team);
        let expected = ValidationResult::invalid(
            "Yikes! Jake Peters has not included players from Burnley. That's gonna sting",
        );

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_pass_if_team_has_players_from_newly_promoted_clubs() {
        let team: Team =
            from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let actual = team_contains_players_from_newly_promoted_clubs(&clubs_by_club_id, &team);
        let expected = ValidationResult::valid();

        assert_eq!(expected, actual)
    }

    #[ignore]
    #[test]
    fn team_to_json() {
        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let gameweek_data: GameweekData =
            from_str(&GAMEWEEK_JSON).expect("Something went wrong parsing gameweek data");

        let picks_data: PicksData = fetch_data_as_json(&format!(
            "https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/",
            986946, 2
        ))
        .expect("Something went wrong parsing picks data");

        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let players_by_player_id = build_players_by_id(&clubs_by_club_id, &bootstrap_data);

        let team = build_team(
            986946,
            "Javier Rufo",
            &players_by_player_id,
            &picks_data,
            &gameweek_data,
        );

        println!(
            "{}",
            serde_json::to_string_pretty(&team).expect("Something went wrong")
        )
    }
}
