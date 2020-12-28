use serde::{Deserialize, Serialize};
use crate::{errors::*, http::client::SHClient};

const PATH: &str = "/v2/groups";
const PATH2: &str = "/timeline";
pub const COUNT: &str = "100";
const ORDER: &str = "asc";

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelineComments {}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelineMessages {
    pub group_id: u32,
    pub id: u32,
    pub is_favorite: bool,
    pub is_silent: bool,
    pub member_id: u32,
    pub published_at: String,
    pub state: String,
    pub text: Option<String>,
    #[serde(rename = "type")]
    pub messages_type: String,
    pub updated_at: String,
    pub file: Option<String>,
    pub thumbnail: Option<String>,
    pub thumbnail_height: Option<u32>,
    pub thumbnail_width: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelineLetters {
    pub client_token: String,
    pub created_at: String,
    pub file: String,
    pub group_id: u32,
    pub id: u32,
    pub is_favorite: bool,
    pub member_id: u32,
    pub opened_at: Option<String>,
    pub text: String,
    pub thumbnail: String,
    pub thumbnail_height: u32,
    pub thumbnail_width: u32,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub comments: Vec<TimelineComments>,
    pub letters: Vec<TimelineLetters>,
    pub messages: Vec<TimelineMessages>,
    pub queried_at: String,
}

pub fn request<C: SHClient>(client: C, access_token: &String, id: &u32, fromdate: &String) -> Result<Timeline> {
    let path = format!("{}/{}{}", PATH, id, PATH2);
    let access_token = String::from(access_token);
    let parameters = vec![
        ("created_from", "2000-01-01T00:00:00Z"),
        ("updated_from", fromdate),
        ("count", COUNT),
        ("order", ORDER)
    ];

    client.get_request::<Timeline>(path.as_str(), &access_token, Some(parameters))
}
