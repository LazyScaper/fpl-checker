use crate::constants::{NEWLY_PROMOTED_CLUBS, VIOLATION_PREFIXES};
use crate::models::{Club, Player, Team, ValidationResult};
use indexmap::IndexMap;
use rand::prelude::IndexedRandom;
use std::collections::HashMap;

pub fn team_contains_players_under_10_m(team: &Team) -> ValidationResult {
    let mut players_above_price_threshold: IndexMap<String, f64> = IndexMap::new();

    for player in &team.players {
        if player.price_in_millions >= 10.0 {
            players_above_price_threshold.insert(player.name.clone(), player.price_in_millions);
        }
    }

    let mut violation_string: String = format!(
        "{} {} has gone overbudget with ",
        VIOLATION_PREFIXES.choose(&mut rand::rng()).unwrap(),
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

    if !players_above_price_threshold.is_empty() {
        return ValidationResult::invalid(&violation_string);
    }

    ValidationResult::valid()
}

pub fn team_contains_at_most_one_player_per_club(team: &Team) -> ValidationResult {
    let mut seen_players_by_club_name: IndexMap<String, Vec<Player>> = IndexMap::new();

    for player in &team.players {
        seen_players_by_club_name
            .entry(player.club.name.clone())
            .or_default()
            .push(player.clone());
    }

    seen_players_by_club_name.retain(|_, players| players.len() > 1);

    let mut violation_string: String = format!(
        "{} {} has",
        VIOLATION_PREFIXES
            .choose(&mut rand::rng())
            .expect("Something went wrong grabbing a prefix"),
        team.owner.clone(),
    );
    for (club_name, players) in &seen_players_by_club_name {
        violation_string.push_str(&format!(" more than 1 player from {} ", club_name));

        for (index, player) in players.iter().enumerate() {
            if index == 0 {
                violation_string.push('(');
            } else if index == players.len() - 1 {
                violation_string.push_str(" and ");
            } else {
                violation_string.push_str(", ");
            }
            violation_string.push_str(&player.name);

            if index == players.len() - 1 {
                violation_string.push(')');
            }
        }
    }

    if !seen_players_by_club_name.is_empty() {
        return ValidationResult::invalid(&violation_string);
    }

    ValidationResult::valid()
}

pub fn team_contains_players_from_newly_promoted_clubs(
    clubs_by_club_id: &HashMap<i64, Club>,
    team: &Team,
) -> ValidationResult {
    for club_id in NEWLY_PROMOTED_CLUBS {
        if !team.players.iter().any(|player| player.club.id == club_id) {
            return ValidationResult::invalid(&format!(
                "{} {} has not included players from {}",
                VIOLATION_PREFIXES.choose(&mut rand::rng()).unwrap(),
                team.owner,
                clubs_by_club_id.get(&club_id).unwrap().name
            ));
        }
    }

    ValidationResult::valid()
}

pub fn run_validators_and_retain_violations(
    clubs_by_club_id: &HashMap<i64, Club>,
    validation_results: &mut Vec<ValidationResult>,
    team: &Team,
) -> Vec<ValidationResult> {
    validation_results.push(team_contains_players_under_10_m(team));
    validation_results.push(team_contains_players_from_newly_promoted_clubs(
        clubs_by_club_id,
        team,
    ));
    validation_results.push(team_contains_at_most_one_player_per_club(team));

    validation_results.retain(|result| !result.is_valid);

    validation_results.clone()
}
