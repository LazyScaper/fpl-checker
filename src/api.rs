use crate::models::{GameweekData, PicksData};

pub fn fetch_data_as_json<T>(uri: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let data = ureq::get(uri).call()?.into_body().read_json::<T>()?;
    Ok(data)
}

pub fn fetch_picks(team_id: &i64, gameweek_data: &GameweekData) -> PicksData {
    let picks_data: PicksData = fetch_data_as_json(&format!(
        "https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/",
        team_id, gameweek_data.current_event
    ))
    .expect("Something went wrong fetching picks data");
    picks_data
}

pub fn fetch_gameweek_data(team_id: &i64) -> GameweekData {
    let gameweek_data: GameweekData = fetch_data_as_json(&format!(
        "https://fantasy.premierleague.com/api/entry/{}/",
        team_id
    ))
    .expect("Something went wrong fetching gameweek data");
    gameweek_data
}