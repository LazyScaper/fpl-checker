use crate::models::{GameweekData, PicksData};
use url::Url;

pub fn fetch_data_as_json<T>(uri: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    // Validate the URL
    validate_fpl_url(uri)?;

    let data = ureq::get(uri).call()?.into_body().read_json::<T>()?;
    Ok(data)
}

pub fn fetch_picks(team_id: &i64, gameweek_data: &GameweekData) -> PicksData {
    // Validate inputs
    validate_team_id(team_id).expect("Invalid team ID");
    validate_gameweek(gameweek_data.current_event).expect("Invalid gameweek");

    let picks_data: PicksData = fetch_data_as_json(&format!(
        "https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/",
        team_id, gameweek_data.current_event
    ))
        .expect("Something went wrong fetching picks data");
    picks_data
}

pub fn fetch_gameweek_data(team_id: &i64) -> GameweekData {
    // Validate team ID
    validate_team_id(team_id).expect("Invalid team ID");

    let gameweek_data: GameweekData = fetch_data_as_json(&format!(
        "https://fantasy.premierleague.com/api/entry/{}/",
        team_id
    ))
        .expect("Something went wrong fetching gameweek data");
    gameweek_data
}

fn validate_team_id(team_id: &i64) -> Result<(), String> {
    if *team_id <= 0 || *team_id > 100_000_000 {
        return Err(format!(
            "Invalid team ID: must be between 1 and 100,000,000, got {}",
            team_id
        ));
    }
    Ok(())
}

fn validate_gameweek(gameweek: i64) -> Result<(), String> {
    if gameweek <= 0 || gameweek > 100 {
        return Err(format!(
            "Invalid gameweek: must be between 1 and 100, got {}",
            gameweek
        ));
    }
    Ok(())
}

fn validate_fpl_url(url_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse(url_str)?;

    if url.host_str() != Some("fantasy.premierleague.com") {
        return Err(format!("Invalid host: {:?}", url.host_str()).into());
    }

    if url.scheme() != "https" {
        return Err(format!("Must use HTTPS, got: {}", url.scheme()).into());
    }

    Ok(())
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Invalid team ID")]
    fn should_reject_negative_team_id() {
        let malicious_team_id = -1;
        fetch_gameweek_data(&malicious_team_id);
    }

    #[test]
    #[should_panic(expected = "Invalid team ID")]
    fn should_reject_unreasonably_large_team_id() {
        let malicious_team_id = 999_999_999;
        fetch_gameweek_data(&malicious_team_id);
    }

    #[test]
    fn should_validate_constructed_url_host() {
        // This would construct:
        // "https://fantasy.premierleague.com/api/entry/1@evil.com/x86/"
        // Some HTTP clients might interpret this as a request to evil.com
        let result = fetch_data_as_json::<GameweekData>(
            "https://fantasy.premierleague.com/api/entry/1@evil.com/x86/",
        );

        assert!(
            result.is_err(),
            "Should reject URL with embedded credentials"
        );
    }

    #[test]
    fn should_reject_url_with_different_host() {
        let result = fetch_data_as_json::<GameweekData>("https://evil.com/malware");

        assert!(result.is_err(), "Should reject non-FPL host");
    }

    #[test]
    fn should_reject_non_https_scheme() {
        let result =
            fetch_data_as_json::<GameweekData>("http://fantasy.premierleague.com/api/entry/1/");

        assert!(result.is_err(), "Should reject HTTP (non-HTTPS) URLs");
    }

    #[test]
    fn should_reject_file_scheme() {
        let result = fetch_data_as_json::<GameweekData>("file:///etc/passwd");

        assert!(result.is_err(), "Should reject file:// URLs");
    }

    #[test]
    #[should_panic(expected = "Invalid team ID")]
    fn should_reject_zero_team_id() {
        fetch_gameweek_data(&0);
    }

    #[test]
    fn should_accept_valid_team_id() {
        // This will fail at the HTTP call level, but should pass validation
        let result = std::panic::catch_unwind(|| {
            validate_team_id(&123456).expect("Should accept valid team ID");
        });

        assert!(result.is_ok(), "Should accept valid team ID");
    }

    #[test]
    fn should_accept_valid_url() {
        let result = validate_fpl_url("https://fantasy.premierleague.com/api/entry/123/");
        assert!(result.is_ok(), "Should accept valid FPL URL");
    }
}