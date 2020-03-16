use serde::{Deserialize, Serialize};
use crate::http;

const PATH: &str = "/subscribe/list";

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeListResultCategories {
    pub category_id: u32,
    pub category_name: String,
    pub sortorder: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeListResult {
    pub arrival: u32,
    pub autorenewing: u32,
    pub categories: Vec<SubscribeListResultCategories>,
    pub destletter: u32,
    pub disable: u32,
    pub enddate: String,
    pub expiredate: String,
    pub fullname: String,
    pub group: String,
    pub groupname: String,
    pub information: String,
    pub lastupdate: String,
    pub price: u32,
    pub productid: String,
    pub remainsec: i32,
    pub subscribed: String,
    pub thumbnail: String,
    pub thumbupdate: String,
    pub trialdays: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeList {
    pub result: Vec<SubscribeListResult>,
    pub status: String,
    pub statuscd: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubscribeListReq {
    token: String,
    username: String,
}

pub fn request(token: &String, username: &String) -> Result<SubscribeList, reqwest::Error> {
    let client = http::Client::new();
    let token = String::from(token);
    let username = String::from(username);
    let subscribe_list_json = SubscribeListReq {token, username};

    client.post_request::<SubscribeList, SubscribeListReq>(PATH, &subscribe_list_json)
}
