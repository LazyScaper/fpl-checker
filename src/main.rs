use serde_json::{from_str, Value};
use std::collections::HashMap;

const PLAYER_AND_TEAM_IDS: [FplTeamInfo; 8] = [
    FplTeamInfo {
        player_name: "Dan",
        team_id: 396409,
    },
    FplTeamInfo {
        player_name: "Jake",
        team_id: 2239760,
    },
    FplTeamInfo {
        player_name: "Jay",
        team_id: 2186577,
    },
    FplTeamInfo {
        player_name: "Shane",
        team_id: 258293,
    },
    FplTeamInfo {
        player_name: "Dylan",
        team_id: 761504,
    },
    FplTeamInfo {
        player_name: "Harry",
        team_id: 7718758,
    },
    FplTeamInfo {
        player_name: "Josh",
        team_id: 2242306,
    },
    FplTeamInfo {
        player_name: "Ed",
        team_id: 8828197,
    },
];

#[derive(Debug, PartialEq, Clone, Default)]

struct ValidationResult {
    is_valid: bool,
    reason: String,
}

#[derive(Debug, PartialEq, Clone, Default)]
struct Player {
    id: i64,
    name: String,
    price_in_millions: f64,
    club_id: i64,
    club: String,
}

#[derive(Debug, PartialEq, Clone)]
struct Team {
    team_id: i64,
    team_name: String,
    owner: String,
    captain: Player,
    players: Vec<Player>,
}
struct FplTeamInfo {
    player_name: &'static str,
    team_id: u32,
}

//
// fn fetch_picked_players(team: &FplTeamInfo) -> Value {
//     let game_week: Response = ureq::get(&format!("https://fantasy.premierleague.com/api/entry/{}/", team.team_id))
//         .call()
//         .unwrap();
//     let game_week: String = game_week.into_string().unwrap();
//     let game_week: GameWeekData = serde_json::from_str(&game_week).unwrap();
//     let picked_players: Response = ureq::get(&format!("https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/", team.team_id, game_week.current_event))
//         .call()
//         .unwrap();
//     let picked_players: String = picked_players.into_string().unwrap();
//     let picked_players: PickDataWrapper = serde_json::from_str(&picked_players).unwrap();
//     picked_players

fn build_teams_by_team_id(bootstrap_data: &str) -> HashMap<i64, String> {
    let bootstrap_data: Value = from_str(bootstrap_data).unwrap();
    let mut teams_by_id: HashMap<i64, String> = HashMap::new();

    if let Some(Value::Array(teams)) = bootstrap_data.get("teams") {
        for team in teams {
            if let Value::Object(team_obj) = team {
                let team_id = match { team_obj.get("id").and_then(|v| v.as_i64()) } {
                    None => panic!("id should be an integer"),
                    Some(id) => id,
                };
                let team_name = match { team_obj.get("name") } {
                    Some(Value::String(first_name)) => first_name.to_string(),
                    _ => panic!("team_obj does not have name"),
                };

                teams_by_id.insert(team_id, team_name);
            }
        }
    }

    teams_by_id
}

fn build_players_by_id(
    teams_by_team_id: &HashMap<i64, String>,
    bootstrap_data: &str,
) -> HashMap<i64, Player> {
    let bootstrap_data: Value = from_str(bootstrap_data).unwrap();
    let mut players_by_id: HashMap<i64, Player> = HashMap::new();

    if let Some(Value::Array(elements)) = bootstrap_data.get("elements") {
        for element in elements {
            if let Value::Object(element_obj) = element {
                let id = match { element_obj.get("id").and_then(|v| v.as_i64()) } {
                    None => panic!("id should be an integer"),
                    Some(id) => id,
                };
                let team_id = match { element_obj.get("team").and_then(|v| v.as_i64()) } {
                    None => panic!("team should be an integer"),
                    Some(team) => team,
                };
                let first_name = match { element_obj.get("first_name") } {
                    Some(Value::String(first_name)) => first_name,
                    _ => panic!("element_obj does not have first_name"),
                };
                let second_name = match { element_obj.get("second_name") } {
                    Some(Value::String(second_name)) => second_name,
                    _ => panic!("element_obj does not have second_name"),
                };
                let price_in_100k = match { element_obj.get("now_cost").and_then(|v| v.as_f64()) } {
                    None => panic!("now_cost should be an integer"),
                    Some(price) => price,
                };
                let price_in_millions: f64 = price_in_100k / 10.0;

                let player = Player {
                    id,
                    name: format!("{} {}", first_name, second_name),
                    price_in_millions,
                    club_id: team_id,
                    club: match { teams_by_team_id.get(&team_id) } {
                        Some(team_name) => team_name.to_string(),
                        _ => {
                            panic!("Could not find a team")
                        }
                    },
                };

                players_by_id.insert(id, player);
            }
        }
    }

    players_by_id
}

fn build_team(
    team_id: i64,
    players_by_player_id: &HashMap<i64, Player>,
    picks_data: &str,
    gameweek_data: &str,
) -> Team {
    let picks_data: Value = from_str(picks_data).unwrap();
    let gameweek_data: Value = from_str(gameweek_data).unwrap();

    let team_name = match { gameweek_data.get("name") } {
        Some(Value::String(team_name)) => team_name,
        _ => panic!("gameweek_data does not have team name"),
    };
    let owner_first_name = match { gameweek_data.get("player_first_name") } {
        Some(Value::String(player_first_name)) => player_first_name,
        _ => panic!("gameweek_data does not have player_first_name"),
    };
    let owner_last_name = match { gameweek_data.get("player_last_name") } {
        Some(Value::String(player_last_name)) => player_last_name,
        _ => panic!("gameweek_data does not have player_last_name"),
    };

    let mut players = Vec::new();
    let mut captain = Player::default();

    if let Some(Value::Array(picks)) = picks_data.get("picks") {
        for pick in picks {
            if let Value::Object(pick_obj) = pick {
                let id = match { pick_obj.get("element").and_then(|v| v.as_i64()) } {
                    None => panic!("element should be an integer"),
                    Some(id) => id,
                };

                let player = Player {
                    id,
                    name: players_by_player_id.get(&id).unwrap().name.clone(),
                    price_in_millions: players_by_player_id.get(&id).unwrap().price_in_millions,
                    club_id: players_by_player_id.get(&id).unwrap().club_id,
                    club: players_by_player_id.get(&id).unwrap().club.to_string(),
                };

                if { pick_obj.get("is_captain").and_then(|v| v.as_bool()) }
                    .unwrap_or_else(|| panic!("is_captain should be a boolean value"))
                {
                    captain = player.clone();
                }

                players.push(player);
            }
        }
    }

    Team {
        team_id: team_id,
        team_name: team_name.to_string(),
        owner: format!("{} {}", owner_first_name, owner_last_name),
        captain,
        players,
    }
}

fn team_contains_players_under_10_m(team: Team) -> ValidationResult {
    let mut players_above_price_threshold: HashMap<String, f64> = HashMap::new();

    for player in team.players {
        if (player.price_in_millions >= 10.0) {
            players_above_price_threshold.insert(player.name, player.price_in_millions);
        }
    }

    for (player_name, price) in players_above_price_threshold {
        return ValidationResult {
            is_valid: false,
            reason: format!(
                "Big wompers! {} has {} in their team. He is currently priced at {}m",
                team.owner,
                player_name,
                price
            ),
        };
    }

    ValidationResult {
        is_valid: true,
        reason: "".to_string(),
    }
}

fn team_contains_at_most_one_player_per_club(team: Team) -> ValidationResult {
    let mut seen_players_by_club_id: HashMap<i64, Player> = HashMap::new();

    for player in team.players {
        if seen_players_by_club_id.contains_key(&player.club_id) {
            return ValidationResult {
                is_valid: false,
                reason: format!(
                    "{} has shat the bed. {} contains more than 1 player from {} ({} and {})",
                    &team.owner,
                    &team.team_name,
                    &player.club,
                    seen_players_by_club_id.get(&player.club_id).unwrap().name,
                    player.name
                ),
            };
        }

        seen_players_by_club_id.insert(player.club_id, player);
    }

    ValidationResult {
        is_valid: true,
        reason: "".to_string(),
    }
}
fn main() {}

mod tests {
    use super::*;

    const BOOTSTRAP_JSON: &str = include_str!("../tests/samples/bootstrap.json");
    const GAMEWEEK_JSON: &str = include_str!("../tests/samples/gameweek.json");
    const PICKS_JSON: &str = include_str!("../tests/samples/picks.json");

    #[test]
    fn should_build_teams_by_team_id_from_bootstrap_data() {
        let actual = build_teams_by_team_id(BOOTSTRAP_JSON);

        assert_eq!(Some(&"Arsenal".to_string()), actual.get(&1));
        assert_eq!(Some(&"Burnley".to_string()), actual.get(&3));
        assert_eq!(Some(&"Brighton".to_string()), actual.get(&6));
        assert_eq!(Some(&"Man City".to_string()), actual.get(&13));
    }

    #[test]
    fn should_build_players_by_id_from_bootstrap_data() {
        let partial_expected = Player {
            id: 249,
            name: "João Pedro Junqueira de Jesus".to_string(),
            price_in_millions: 7.5,
            club_id: 7,
            club: "Chelsea".to_string(),
        };
        let teams_by_team_id = build_teams_by_team_id(BOOTSTRAP_JSON);
        let actual = build_players_by_id(&teams_by_team_id, BOOTSTRAP_JSON);

        assert_eq!(Some(&partial_expected), actual.get(&partial_expected.id));
    }

    #[test]
    fn should_build_team_from_data() {
        let expected = Team {
            team_id: 2239760,
            team_name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 249,
                name: "João Pedro Junqueira de Jesus".to_string(),
                price_in_millions: 7.5,
                club_id: 7,
                club: "Chelsea".to_string(),
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Jordan Pickford".to_string(),
                    price_in_millions: 5.5,
                    club_id: 9,
                    club: "Everton".to_string(),
                },
                Player {
                    id: 145,
                    name: "Maxim De Cuyper".to_string(),
                    price_in_millions: 4.5,
                    club_id: 6,
                    club: "Brighton".to_string(),
                },
                Player {
                    id: 506,
                    name: "Murillo Costa dos Santos".to_string(),
                    price_in_millions: 5.5,
                    club_id: 16,
                    club: "Nott'm Forest".to_string(),
                },
                Player {
                    id: 348,
                    name: "Joe Rodon".to_string(),
                    price_in_millions: 4.0,
                    club_id: 11,
                    club: "Leeds".to_string(),
                },
                Player {
                    id: 119,
                    name: "Bryan Mbeumo".to_string(),
                    price_in_millions: 8.0,
                    club_id: 14,
                    club: "Man Utd".to_string(),
                },
                Player {
                    id: 382,
                    name: "Florian Wirtz".to_string(),
                    price_in_millions: 8.5,
                    club_id: 12,
                    club: "Liverpool".to_string(),
                },
                Player {
                    id: 413,
                    name: "Omar Marmoush".to_string(),
                    price_in_millions: 8.5,
                    club_id: 13,
                    club: "Man City".to_string(),
                },
                Player {
                    id: 582,
                    name: "Mohammed Kudus".to_string(),
                    price_in_millions: 6.5,
                    club_id: 18,
                    club: "Spurs".to_string(),
                },
                Player {
                    id: 666,
                    name: "Viktor Gyökeres".to_string(),
                    price_in_millions: 9.0,
                    club_id: 1,
                    club: "Arsenal".to_string(),
                },
                Player {
                    id: 249,
                    name: "João Pedro Junqueira de Jesus".to_string(),
                    price_in_millions: 7.5,
                    club_id: 7,
                    club: "Chelsea".to_string(),
                },
                Player {
                    id: 624,
                    name: "Jarrod Bowen".to_string(),
                    price_in_millions: 8.0,
                    club_id: 19,
                    club: "West Ham".to_string(),
                },
                Player {
                    id: 470,
                    name: "Martin Dúbravka".to_string(),
                    price_in_millions: 4.0,
                    club_id: 3,
                    club: "Burnley".to_string(),
                },
                Player {
                    id: 486,
                    name: "Anthony Elanga".to_string(),
                    price_in_millions: 7.0,
                    club_id: 15,
                    club: "Newcastle".to_string(),
                },
                Player {
                    id: 541,
                    name: "Reinildo Mandava".to_string(),
                    price_in_millions: 4.0,
                    club_id: 17,
                    club: "Sunderland".to_string(),
                },
                Player {
                    id: 256,
                    name: "Daniel Muñoz Mejía".to_string(),
                    price_in_millions: 5.5,
                    club_id: 8,
                    club: "Crystal Palace".to_string(),
                },
            ],
        };
        let teams_by_team_id = build_teams_by_team_id(BOOTSTRAP_JSON);
        let players_by_player_id = build_players_by_id(&teams_by_team_id.clone(), BOOTSTRAP_JSON);
        let actual = build_team(2239760, &players_by_player_id, PICKS_JSON, GAMEWEEK_JSON);

        assert_eq!(expected, actual);
    }

    #[test]
    fn should_fail_if_team_has_more_than_one_player_from_a_club() {
        let team = Team {
            team_id: 2239760,
            team_name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 287,
                name: "Jordan Pickford".to_string(),
                price_in_millions: 7.5,
                club_id: 9,
                club: "Everton".to_string(),
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Jordan Pickford".to_string(),
                    price_in_millions: 7.5,
                    club_id: 9,
                    club: "Everton".to_string(),
                },
                Player {
                    id: 291,
                    name: "James Tarkowski".to_string(),
                    price_in_millions: 5.5,
                    club_id: 9,
                    club: "Everton".to_string(),
                },
            ],
        };

        let actual = team_contains_at_most_one_player_per_club(team);
        let expected = ValidationResult { is_valid: false, reason: "Jake Peters has shat the bed. Pedro Cask Ale contains more than 1 player from Everton (Jordan Pickford and James Tarkowski)".to_string() };

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_pass_if_team_does_not_have_more_than_one_player_from_a_club() {
        let team = Team {
            team_id: 2239760,
            team_name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 249,
                name: "João Pedro Junqueira de Jesus".to_string(),
                price_in_millions: 7.5,
                club_id: 7,
                club: "Chelsea".to_string(),
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Jordan Pickford".to_string(),
                    price_in_millions: 5.5,
                    club_id: 9,
                    club: "Everton".to_string(),
                },
                Player {
                    id: 145,
                    name: "Maxim De Cuyper".to_string(),
                    price_in_millions: 4.5,
                    club_id: 6,
                    club: "Brighton".to_string(),
                },
                Player {
                    id: 506,
                    name: "Murillo Costa dos Santos".to_string(),
                    price_in_millions: 5.5,
                    club_id: 16,
                    club: "Nott'm Forest".to_string(),
                },
                Player {
                    id: 348,
                    name: "Joe Rodon".to_string(),
                    price_in_millions: 4.0,
                    club_id: 11,
                    club: "Leeds".to_string(),
                },
                Player {
                    id: 119,
                    name: "Bryan Mbeumo".to_string(),
                    price_in_millions: 8.0,
                    club_id: 14,
                    club: "Man Utd".to_string(),
                },
                Player {
                    id: 382,
                    name: "Florian Wirtz".to_string(),
                    price_in_millions: 8.5,
                    club_id: 12,
                    club: "Liverpool".to_string(),
                },
                Player {
                    id: 413,
                    name: "Omar Marmoush".to_string(),
                    price_in_millions: 8.4,
                    club_id: 13,
                    club: "Man City".to_string(),
                },
                Player {
                    id: 582,
                    name: "Mohammed Kudus".to_string(),
                    price_in_millions: 6.5,
                    club_id: 18,
                    club: "Spurs".to_string(),
                },
                Player {
                    id: 666,
                    name: "Viktor Gyökeres".to_string(),
                    price_in_millions: 9.0,
                    club_id: 1,
                    club: "Arsenal".to_string(),
                },
                Player {
                    id: 249,
                    name: "João Pedro Junqueira de Jesus".to_string(),
                    price_in_millions: 7.5,
                    club_id: 7,
                    club: "Chelsea".to_string(),
                },
                Player {
                    id: 624,
                    name: "Jarrod Bowen".to_string(),
                    price_in_millions: 8.0,
                    club_id: 19,
                    club: "West Ham".to_string(),
                },
                Player {
                    id: 470,
                    name: "Martin Dúbravka".to_string(),
                    price_in_millions: 4.0,
                    club_id: 3,
                    club: "Burnley".to_string(),
                },
                Player {
                    id: 486,
                    name: "Anthony Elanga".to_string(),
                    price_in_millions: 7.0,
                    club_id: 15,
                    club: "Newcastle".to_string(),
                },
                Player {
                    id: 541,
                    name: "Reinildo Mandava".to_string(),
                    price_in_millions: 4.0,
                    club_id: 17,
                    club: "Sunderland".to_string(),
                },
                Player {
                    id: 256,
                    name: "Daniel Muñoz Mejía".to_string(),
                    price_in_millions: 5.5,
                    club_id: 8,
                    club: "Crystal Palace".to_string(),
                },
            ],
        };

        let actual = team_contains_at_most_one_player_per_club(team);
        let expected = ValidationResult {
            is_valid: true,
            reason: "".to_string(),
        };

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_fail_if_team_has_player_above_price_limit() {
        let team = Team {
            team_id: 2239760,
            team_name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 287,
                name: "Jordan Pickford".to_string(),
                price_in_millions: 7.5,
                club_id: 9,
                club: "Everton".to_string(),
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Jordan Pickford".to_string(),
                    price_in_millions: 7.5,
                    club_id: 9,
                    club: "Everton".to_string(),
                },
                Player {
                    id: 291,
                    name: "James Tarkowski".to_string(),
                    price_in_millions: 10.5,
                    club_id: 9,
                    club: "Everton".to_string(),
                },
            ],
        };

        let actual = team_contains_players_under_10_m(team);
        let expected = ValidationResult { is_valid: false, reason: "Big wompers! Jake Peters has James Tarkowski in their team. He is currently priced at 10.5m".to_string() };

        assert_eq!(expected, actual)
    }

    #[test]
    fn should_pass_if_team_has_players_under_price_limit() {
        let team = Team {
            team_id: 2239760,
            team_name: "Pedro Cask Ale".to_string(),
            owner: "Jake Peters".to_string(),
            captain: Player {
                id: 249,
                name: "João Pedro Junqueira de Jesus".to_string(),
                price_in_millions: 7.5,
                club_id: 7,
                club: "Chelsea".to_string(),
            },
            players: vec![
                Player {
                    id: 287,
                    name: "Jordan Pickford".to_string(),
                    price_in_millions: 5.5,
                    club_id: 9,
                    club: "Everton".to_string(),
                },
                Player {
                    id: 145,
                    name: "Maxim De Cuyper".to_string(),
                    price_in_millions: 4.5,
                    club_id: 6,
                    club: "Brighton".to_string(),
                },
                Player {
                    id: 506,
                    name: "Murillo Costa dos Santos".to_string(),
                    price_in_millions: 5.5,
                    club_id: 16,
                    club: "Nott'm Forest".to_string(),
                },
                Player {
                    id: 348,
                    name: "Joe Rodon".to_string(),
                    price_in_millions: 4.0,
                    club_id: 11,
                    club: "Leeds".to_string(),
                },
                Player {
                    id: 119,
                    name: "Bryan Mbeumo".to_string(),
                    price_in_millions: 8.0,
                    club_id: 14,
                    club: "Man Utd".to_string(),
                },
                Player {
                    id: 382,
                    name: "Florian Wirtz".to_string(),
                    price_in_millions: 8.5,
                    club_id: 12,
                    club: "Liverpool".to_string(),
                },
                Player {
                    id: 413,
                    name: "Omar Marmoush".to_string(),
                    price_in_millions: 8.4,
                    club_id: 13,
                    club: "Man City".to_string(),
                },
                Player {
                    id: 582,
                    name: "Mohammed Kudus".to_string(),
                    price_in_millions: 6.5,
                    club_id: 18,
                    club: "Spurs".to_string(),
                },
                Player {
                    id: 666,
                    name: "Viktor Gyökeres".to_string(),
                    price_in_millions: 9.0,
                    club_id: 1,
                    club: "Arsenal".to_string(),
                },
                Player {
                    id: 249,
                    name: "João Pedro Junqueira de Jesus".to_string(),
                    price_in_millions: 7.5,
                    club_id: 7,
                    club: "Chelsea".to_string(),
                },
                Player {
                    id: 624,
                    name: "Jarrod Bowen".to_string(),
                    price_in_millions: 8.0,
                    club_id: 19,
                    club: "West Ham".to_string(),
                },
                Player {
                    id: 470,
                    name: "Martin Dúbravka".to_string(),
                    price_in_millions: 4.0,
                    club_id: 3,
                    club: "Burnley".to_string(),
                },
                Player {
                    id: 486,
                    name: "Anthony Elanga".to_string(),
                    price_in_millions: 7.0,
                    club_id: 15,
                    club: "Newcastle".to_string(),
                },
                Player {
                    id: 541,
                    name: "Reinildo Mandava".to_string(),
                    price_in_millions: 4.0,
                    club_id: 17,
                    club: "Sunderland".to_string(),
                },
                Player {
                    id: 256,
                    name: "Daniel Muñoz Mejía".to_string(),
                    price_in_millions: 5.5,
                    club_id: 8,
                    club: "Crystal Palace".to_string(),
                },
            ],
        };

        let actual = team_contains_players_under_10_m(team);
        let expected = ValidationResult {
            is_valid: true,
            reason: "".to_string(),
        };

        assert_eq!(expected, actual)
    }

}
