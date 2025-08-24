use crate::models::{BootstrapData, Club, GameweekData, PicksData, Player, Team};
use std::collections::HashMap;

pub fn build_team_from_data(
    team_id: i64,
    players_by_player_id: &HashMap<i64, Player>,
    gameweek_data: &GameweekData,
    picks_data: &PicksData,
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
        name: gameweek_data.name.clone(),
        owner: gameweek_data.player_first_name.clone(),
        captain,
        players,
    }
}

pub fn build_clubs_by_id(bootstrap_data: &BootstrapData) -> HashMap<i64, String> {
    let mut clubs_by_id: HashMap<i64, String> = HashMap::new();

    for club in &bootstrap_data.teams {
        clubs_by_id.insert(club.id, club.name.clone());
    }

    clubs_by_id
}

pub fn build_players_by_id(
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

pub fn fetch_and_build_team(team_id: i64, players_by_player_id: &HashMap<i64, Player>) -> Team {
    let gameweek_data: GameweekData = crate::api::fetch_data_as_json(&format!(
        "https://fantasy.premierleague.com/api/entry/{}/",
        team_id
    ))
    .expect("Something went wrong fetching gameweek data");

    let picks_data: PicksData = crate::api::fetch_data_as_json(&format!(
        "https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/",
        team_id, gameweek_data.current_event
    ))
    .expect("Something went wrong fetching picks data");

    build_team_from_data(team_id, players_by_player_id, &gameweek_data, &picks_data)
}
