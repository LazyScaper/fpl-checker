use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BootstrapTeam {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
pub struct BootstrapEvent {
    pub id: i64,
    pub is_current: bool,
}

#[derive(Deserialize)]
pub struct BootstrapElement {
    pub id: i64,
    pub web_name: String,
    pub now_cost: f64,
    pub team: i64,
}

#[derive(Deserialize)]
pub struct BootstrapData {
    pub elements: Vec<BootstrapElement>,
    pub events: Vec<BootstrapEvent>,
    pub teams: Vec<BootstrapTeam>,
}

#[derive(Deserialize)]
pub struct PicksData {
    pub picks: Vec<PickElement>,
}

#[derive(Deserialize)]
pub struct PickElement {
    pub is_captain: bool,
    pub element: i64,
}

#[derive(Deserialize)]
pub struct GameweekData {
    pub current_event: i64,
    pub name: String,
    pub player_first_name: String,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub reason: String,
}

impl ValidationResult {
    pub(crate) fn valid() -> Self {
        Self {
            is_valid: true,
            reason: "".to_string(),
        }
    }

    pub(crate) fn invalid(reason: &str) -> Self {
        Self {
            is_valid: false,
            reason: reason.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Club {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Player {
    pub id: i64,
    pub name: String,
    pub price_in_millions: f64,
    pub club: Club,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub owner: String,
    pub captain: Player,
    pub players: Vec<Player>,
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct TeamsRequest {
    pub teams: Vec<i64>,
}