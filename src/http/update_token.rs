use serde::{Deserialize, Serialize};
use crate::http;
use crate::errors::*;

const PATH: &str = "/v2/update_token";

#[derive(Serialize, Deserialize, Debug)]
pub struct InvalidParameter {
    pub code: String,
    pub message: String,
    pub parameter: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateToken {
    pub access_token: String,
    pub expires_in: u32,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTokenReq {
    pub refresh_token: String
}

pub fn request(refresh_token: &String) -> Result<UpdateToken> {
    let client = http::Client::new();
    let refresh_token = String::from(refresh_token);

    let update_token_json = UpdateTokenReq { refresh_token };
    client.post_request::<UpdateToken, UpdateTokenReq>(PATH, &update_token_json)
}
