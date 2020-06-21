use serde::{Deserialize, Serialize};
use crate::http;
use crate::errors::*;

const PATH: &str = "/v2/groups";

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupsSubscription {
    pub auto_renewing: bool,
    pub end_at: Option<String>,
    pub start_at: String,
    #[serde(rename = "type")]
    pub subscription_type: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Groups {
    pub id: u32,
    pub is_letter_destination: bool,
    pub name: String,
    pub phone_image: Option<String>,
    pub priority: u32,
    pub state: String,
    pub tags: Vec<String>,
    pub thumbnail: String,
    pub trial_days: Option<u32>,
    pub updated_at: String,
    pub subscription: Option<GroupsSubscription>
}

pub fn request(access_token: &String) -> Result<Vec<Groups>> {
    let client = http::Client::new();
    let access_token = String::from(access_token);

    client.get_request::<Vec<Groups>>(PATH,  &access_token, None)
}
