use serde::{Deserialize, Serialize};
use crate::{errors::*, http::client::SHNClient};

const PATH: &str = "/v2/groups";
const PATH2: &str = "/members";

#[derive(Serialize, Deserialize, Debug)]
pub struct Members {
    pub id: u32,
    pub name: String,
}

pub fn request<C: SHNClient>(client: C, access_token: &String, group_id: &u32) -> Result<Vec<Members>> {
    let path = format!("{}/{}{}", PATH, group_id, PATH2);
    let access_token = String::from(access_token);

    client.get_request::<Vec<Members>>(path.as_str(), &access_token, None, false)
}
