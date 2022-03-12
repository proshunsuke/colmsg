use serde::{Deserialize, Serialize};
use crate::{errors::*, http::client::SHNClient, http::timeline::TimelineMessages};

const PATH: &str = "/v2/groups";
const PATH2: &str = "/past_messages";
const ORDER: &str = "asc";

#[derive(Serialize, Deserialize, Debug)]
pub struct PastMessages {
    pub messages: Vec<TimelineMessages>,
}

pub fn request<C: SHNClient>(client: C, access_token: &String, id: &u32) -> Result<PastMessages> {
    let path = format!("{}/{}{}", PATH, id, PATH2);
    let access_token = String::from(access_token);
    let parameters = vec![
        ("order", ORDER)
    ];

    client.get_request::<PastMessages>(path.as_str(), &access_token, Some(parameters), true)
}
