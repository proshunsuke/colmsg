use serde::{Deserialize, Serialize};
use crate::{errors::*, http::client::SHNClient};

const PATH: &str = "/v2/tags";

#[derive(Serialize, Deserialize, Debug)]
pub struct TagsMeta {
    pub color: Option<String>, // memo: khは必須だがには無い
    pub dimension: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tags {
    pub meta: Option<TagsMeta>, // memo: khは必須だがには無い場合がある
    pub name: String,
    pub priority: u32,
    pub updated_at: String,
    pub uuid: String,
}

pub fn request<C: SHNClient>(client: C, access_token: &String) -> Result<Vec<Tags>> {
    let access_token = String::from(access_token);

    client.get_request::<Vec<Tags>>(PATH, &access_token, None, false)
}
