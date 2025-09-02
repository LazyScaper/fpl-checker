use crate::api::{fetch_gameweek_data, fetch_picks};
use crate::builders::build_team_from_data;
use crate::models::TeamsRequest;
use constants::BOOTSTRAP_DATA_URI;
use models::{BootstrapData, ValidationResult};
use rocket::serde::json::Json;
use rocket::{build, post, routes, Build, Rocket};

mod api;
mod builders;
mod constants;
mod models;
mod validators;

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().collect();

    if arguments.len() < 2 {
        print_usage();
        return;
    }

    if arguments[1] == "--api" {
        let _ = build_rocket().launch().await;
    } else {
        let team_ids: Vec<i64> = parse_team_ids_from_cli();
        let violations = run_validation_for_teams(team_ids);

        println!("{}", process_validation_results(violations));
    }
}

#[post("/api", data = "<input>")]
fn handle_teams_request(input: Json<TeamsRequest>) -> Json<String> {
    let violations = run_validation_for_teams(input.teams.clone());
    Json(process_validation_results(violations))
}

fn build_rocket() -> Rocket<Build> {
    build().mount("/", routes![handle_teams_request])
}

fn run_validation_for_teams(team_ids: Vec<i64>) -> Vec<ValidationResult> {
    let bootstrap_data: BootstrapData = api::fetch_data_as_json(BOOTSTRAP_DATA_URI)
        .expect("Something went wrong fetching bootstrap data");
    let clubs_by_club_id = builders::build_clubs_by_id(&bootstrap_data);
    let players_by_id = builders::build_players_by_id(&clubs_by_club_id, &bootstrap_data);
    let current_gameweek = builders::get_current_gameweek(&bootstrap_data);

    println!("Checking gameweek {}...", current_gameweek);

    let mut validation_results: Vec<ValidationResult> = Vec::new();
    let mut violations: Vec<ValidationResult> = Vec::new();

    for fpl_team_id in team_ids {
        let gameweek_data = fetch_gameweek_data(&fpl_team_id);
        let picks_data = fetch_picks(&fpl_team_id, &gameweek_data);
        let team = build_team_from_data(fpl_team_id, &players_by_id, &gameweek_data, &picks_data);

        violations.extend(validators::run_validators_and_retain_violations(
            &clubs_by_club_id,
            &mut validation_results,
            &team,
        ));
    }

    violations
}

fn process_validation_results(violations: Vec<ValidationResult>) -> String {
    if violations.is_empty() {
        return "No rules have been broken... boring!".to_string();
    }

    let mut output = String::new();
    for validation in violations {
        if !validation.is_valid {
            output.push_str(&format!("{}", validation.reason + "\n\n"))
        }
    }

    output
}

fn parse_team_ids_from_cli() -> Vec<i64> {
    std::env::args()
        .skip(1)
        .map(|arg| {
            arg.parse::<i64>()
                .unwrap_or_else(|_| panic!("Invalid team ID: {}", arg))
        })
        .collect()
}

fn print_usage() {
    println!("Usage: fpl-checker <team_id> [<team_id> ...]");
    println!("       fpl-checker --api");
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
    use assertor::*;
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

        assert_that!(&"Arsenal".to_string())
            .is_equal_to(&actual.get(&1).expect("Club not found").name);
        assert_that!(&"Burnley".to_string())
            .is_equal_to(&actual.get(&3).expect("Club not found").name);
        assert_that!(&"Brighton".to_string())
            .is_equal_to(&actual.get(&6).expect("Club not found").name);
        assert_that!(&"Man City".to_string())
            .is_equal_to(&actual.get(&13).expect("Club not found").name);
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

        assert_that!(actual.get(&partial_expected.id)).is_equal_to(Some(&partial_expected));
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

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn should_fail_if_team_has_more_than_one_player_from_a_club() {
        let team = from_str(&INVALID_TEAM_DUPLICATE_ARSENAL_JSON)
            .expect("Something went wrong parsing invalid team");
        let actual = team_contains_at_most_one_player_per_club(&team);

        assert_that!(actual.reason)
            .contains("has more than 1 player from Arsenal (Gabriel and Gyökeres)");
    }

    #[test]
    fn should_fail_if_team_has_more_than_a_few_players_from_multiple_clubs() {
        let team = from_str(&INVALID_TEAM_MANY_PLAYERS_MANY_CLUBS_JSON)
            .expect("Something went wrong parsing invalid team");
        let actual = team_contains_at_most_one_player_per_club(&team);

        assert_that!(actual.reason)
            .contains("has more than 1 player from Chelsea (Sánchez and João Pedro) more than 1 player from Arsenal (Gabriel, Saliba and Gyökeres) more than 1 player from Man Utd (Yoro and Mbeumo)");
    }

    #[test]
    fn should_pass_if_team_does_not_have_more_than_one_player_from_a_club() {
        let team: Team =
            from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let actual = team_contains_at_most_one_player_per_club(&team);
        let expected = ValidationResult::valid();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn should_fail_if_team_has_player_above_price_limit() {
        let team = from_str(&INVALID_TEAM_MISSING_PLAYER_OVER_10M)
            .expect("Something went wrong parsing invalid team");
        let actual = team_contains_players_under_10_m(&team);

        assert_that!(actual.reason).contains("has gone overbudget with Haaland (14m)");
    }

    #[test]
    fn should_produce_multiple_failures_if_team_has_more_than_1_player_above_price_limit() {
        let team = from_str(&INVALID_TEAM_JSON).expect("Something went wrong parsing invalid team");
        let actual = team_contains_players_under_10_m(&team);

        assert_that!(actual.reason)
            .contains("has gone overbudget with Palmer (10.5m) and Haaland (14m)");
    }

    #[test]
    fn should_pass_if_team_has_players_under_price_limit() {
        let team: Team =
            from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let actual = team_contains_players_under_10_m(&team);
        let expected = ValidationResult::valid();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn should_fail_if_team_does_not_have_players_from_newly_promoted_clubs() {
        let team = from_str(INVALID_TEAM_MISSING_BURNLEY)
            .expect("Something went wrong parsing invalid team");

        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let actual = team_contains_players_from_newly_promoted_clubs(&clubs_by_club_id, &team);

        assert_that!(actual.reason).contains("has not included players from Burnley")
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

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn should_pass_all_validation_rules_if_valid() {
        let bootstrap_data: BootstrapData =
            from_str(&BOOTSTRAP_JSON).expect("Something went wrong parsing bootstrap data");
        let clubs_by_club_id = build_clubs_by_id(&bootstrap_data);
        let team = from_str(&VALID_TEAM_JSON).expect("Something went wrong parsing valid team");
        let mut validation_results: Vec<ValidationResult> = Vec::new();

        let violations = validators::run_validators_and_retain_violations(
            &clubs_by_club_id,
            &mut validation_results,
            &team,
        );

        assert_that!(violations).is_empty()
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
