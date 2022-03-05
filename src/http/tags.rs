use serde::{Deserialize, Serialize};
use crate::{errors::*, http::client::SHClient};

const PATH: &str = "/v2/tags";

#[derive(Serialize, Deserialize, Debug)]
pub struct TagsMeta {
    pub color: String,
    pub dimension: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tags {
    pub meta: TagsMeta,
    pub name: String,
    pub priority: u32,
    pub updated_at: String,
    pub uuid: String,
}

pub fn request<C: SHClient>(client: C, access_token: &String) -> Result<Vec<Tags>> {
    let access_token = String::from(access_token);

    client.get_request::<Vec<Tags>>(PATH, &access_token, None, false)
}
