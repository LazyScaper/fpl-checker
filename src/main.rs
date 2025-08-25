use crate::api::{fetch_gameweek_data, fetch_picks};
use crate::builders::build_team_from_data;
use constants::BOOTSTRAP_DATA_URI;
use models::{BootstrapData, ValidationResult};
use std::ops::Not;

mod api;
mod builders;
mod constants;
mod models;
mod validators;

fn main() {
    if std::env::args().len() < 2 {
        println!("Usage: fpl-checker <team_id> [<team_id> ...]");
        return;
    }

    let team_ids: Vec<i64> = std::env::args()
        .skip(1)
        .map(|arg| {
            arg.parse::<i64>()
                .expect(&format!("Invalid team ID: {}", arg))
        })
        .collect();

    let bootstrap_data: BootstrapData = api::fetch_data_as_json(BOOTSTRAP_DATA_URI)
        .expect("Something went wrong fetching bootstrap data");
    let clubs_by_club_id = builders::build_clubs_by_id(&bootstrap_data);
    let players_by_id = builders::build_players_by_id(&clubs_by_club_id, &bootstrap_data);

    let mut validation_results: Vec<ValidationResult> = Vec::new();
    let mut violations: Vec<ValidationResult> = Vec::new();
    let mut current_gameweek = -1;

    for fpl_team_id in team_ids {
        let gameweek_data = fetch_gameweek_data(&fpl_team_id);
        let picks_data = fetch_picks(&fpl_team_id, &gameweek_data);
        let team = build_team_from_data(fpl_team_id, &players_by_id, &gameweek_data, &picks_data);

        current_gameweek = gameweek_data.current_event;

        violations.extend(validators::run_and_retain_violations(
            &clubs_by_club_id,
            &mut validation_results,
            &team,
        ));
    }

    if violations.is_empty() {
        println!("No rules have been broken for gameweek {}!", current_gameweek);
        return;
    }

    for validation in violations {
        if validation.is_valid.not() {
            println!("{}\n\n", validation.reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::fetch_data_as_json;
    use crate::builders::{build_clubs_by_id, build_players_by_id, build_team_from_data};
    use crate::models::{Club, GameweekData, PicksData, Player, Team};
    use crate::validators::{
        team_contains_at_most_one_player_per_club, team_contains_players_from_newly_promoted_clubs,
        team_contains_players_under_10_m,
    };
    use serde_json::from_str;

    const BOOTSTRAP_JSON: &str = include_str!("../tests/samples/bootstrap.json");
    const GAMEWEEK_JSON: &str = include_str!("../tests/samples/gameweek.json");
    const PICKS_JSON: &str = include_str!("../tests/samples/picks.json");
    const VALID_TEAM_JSON: &str = include_str!("../tests/samples/valid_team.json");
    const INVALID_TEAM_JSON: &str = include_str!("../tests/samples/invalid_team.json");
    const INVALID_TEAM_DUPLICATE_ARSENAL_JSON: &str =
        include_str!("../tests/samples/invalid_team_duplicate_arsenal.json");
    const INVALID_TEAM_MANY_PLAYERS_MANY_CLUBS_JSON: &str =
        include_str!("../tests/samples/invalid_team_many_players_many_clubs.json");
    const INVALID_TEAM_MISSING_PLAYER_OVER_10M: &str =
        include_str!("../tests/samples/invalid_team_missing_player_over_10m.json");
    const INVALID_TEAM_MISSING_BURNLEY: &str =
        include_str!("../tests/samples/invalid_team_missing_burnley.json");

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
        let expected: Team =
            from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");

        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let gameweek_data: GameweekData =
            from_str(&GAMEWEEK_JSON).expect("Something went wrong parsing gameweek data");
        let picks_data: PicksData =
            from_str(&PICKS_JSON).expect("Something went wrong parsing picks data");

        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let players_by_player_id = build_players_by_id(&clubs_by_club_id.clone(), &bootstrap_data);
        let actual =
            build_team_from_data(2239760, &players_by_player_id, &gameweek_data, &picks_data);

        assert_eq!(actual, expected);
    }

    #[test]
    fn should_fail_if_team_has_more_than_one_player_from_a_club() {
        let team = from_str(&INVALID_TEAM_DUPLICATE_ARSENAL_JSON)
            .expect("Something went wrong parsing invalid team");
        let actual = team_contains_at_most_one_player_per_club(&team);
        let expected = ValidationResult::invalid(
            "Jake has shat the bed. Pedro Cask Ale contains more than 1 player from Arsenal (Gabriel and Gyökeres)",
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_fail_if_team_has_more_than_a_few_players_from_multiple_clubs() {
        let team = from_str(&INVALID_TEAM_MANY_PLAYERS_MANY_CLUBS_JSON)
            .expect("Something went wrong parsing invalid team");
        let actual = team_contains_at_most_one_player_per_club(&team);
        let expected = ValidationResult::invalid(
            "Jake has shat the bed. Pedro Cask Ale contains more than 1 player from Chelsea (Sánchez and João Pedro) more than 1 player from Arsenal (Gabriel, Saliba and Gyökeres) more than 1 player from Man Utd (Yoro and Mbeumo)",
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_pass_if_team_does_not_have_more_than_one_player_from_a_club() {
        let team: Team =
            from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let actual = team_contains_at_most_one_player_per_club(&team);
        let expected = ValidationResult::valid();

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_fail_if_team_has_player_above_price_limit() {
        let team = from_str(&INVALID_TEAM_MISSING_PLAYER_OVER_10M)
            .expect("Something went wrong parsing invalid team");
        let actual = team_contains_players_under_10_m(&team);
        let expected =
            ValidationResult::invalid("Big wompers! Jake has gone overbudget with Haaland (14m)");

        assert_eq!(actual, expected)
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

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_fail_if_team_does_not_have_players_from_newly_promoted_clubs() {
        let team = from_str(INVALID_TEAM_MISSING_BURNLEY)
            .expect("Something went wrong parsing invalid team");

        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let actual = team_contains_players_from_newly_promoted_clubs(&clubs_by_club_id, &team);
        let expected = ValidationResult::invalid(
            "Yikes! Javier Rufo has not included players from Burnley. That's gonna sting",
        );

        assert_eq!(actual, expected)
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

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_pass_all_validation_rules_if_valid() {
        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let team = from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let mut validation_results: Vec<ValidationResult> = Vec::new();

        let violations = validators::run_and_retain_violations(
            &clubs_by_club_id,
            &mut validation_results,
            &team,
        );

        assert!(violations.is_empty())
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
            866231, 2
        ))
        .expect("Something went wrong parsing picks data");

        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let players_by_player_id = build_players_by_id(&clubs_by_club_id, &bootstrap_data);

        let team =
            build_team_from_data(4402816, &players_by_player_id, &gameweek_data, &picks_data);

        println!(
            "{}",
            serde_json::to_string_pretty(&team).expect("Something went wrong")
        )
    }
}
