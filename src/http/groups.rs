use serde::{Deserialize, Serialize};
use crate::{errors::*, http::client::SHNClient};

const PATH: &str = "/v2/groups";

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupsSubscription {
    pub auto_renewing: bool,
    pub end_at: Option<String>,
    pub start_at: String,
    #[serde(rename = "type")]
    pub subscription_type: String,
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
    pub subscription: Option<GroupsSubscription>,
}

pub fn request<C: SHNClient>(client: C, access_token: &String) -> Result<Vec<Groups>> {
    let access_token = String::from(access_token);

    client.get_request::<Vec<Groups>>(PATH, &access_token, None, false)
}
