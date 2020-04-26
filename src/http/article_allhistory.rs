use serde::{Deserialize, Serialize};
use crate::http;

const PATH: &str = "/article/allhistory";

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistoryResultGroupMembers {
    pub author: String,
    pub destletter: u32,
    pub disable: u32,
    pub lastupdate: String,
    pub name: String,
    pub sortorder: u32,
    pub thumbupdate: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistoryResultGroup {
    pub autorenewing: u32,
    pub destletter: u32,
    pub disable: u32,
    pub enddate: String,
    pub expiredate: String,
    pub lastupdate: String,
    pub members: Vec<ArticleAllhistoryResultGroupMembers>,
    pub remainsec: i32,
    pub subscribed: String,
    pub thumbupdate: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistoryResultHistoryBodyLinkParam {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistoryResultHistoryBody {
    pub author: String,
    pub contents: Option<String>,
    pub date: String,
    pub disable: u32,
    pub favorite: bool,
    pub group: String,
    pub link_param: Option<ArticleAllhistoryResultHistoryBodyLinkParam>,
    pub media: Option<u32>,
    pub parent_comment: Option<String>,
    pub parent_fanletter: Option<String>,
    pub premium: Option<u32>,
    pub seq_id: Option<u32>,
    pub special: Option<u32>,
    pub stamp: Option<String>,
    pub talk: Option<String>,
    pub thread: String,
    pub thumbheight: u32,
    pub thumbwidth: u32,
    pub lastupdate: Option<String>,
    pub letter: Option<String>,
    pub opendate: Option<String>,
    pub opened: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistoryResultHistory {
    pub body: ArticleAllhistoryResultHistoryBody,
    pub kind: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistoryResult {
    pub group: ArticleAllhistoryResultGroup,
    pub history: Vec<ArticleAllhistoryResultHistory>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistory {
    pub result: ArticleAllhistoryResult,
    pub status: String,
    pub statuscd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleAllhistoryReq {
    pub group: String,
    pub token: String,
    pub username: String,
    pub count: u32,
    pub fromdate: String,
    pub sortorder: u32,
    pub todate: String,
}

pub fn request(
    group: &String,
    token: &String,
    username: &String,
    count: u32,
    fromdate: &String,
    sortorder: u32,
    todate: &String,
) -> Result<ArticleAllhistory, reqwest::Error> {
    let client = http::Client::new();
    let group = String::from(group);
    let token = String::from(token);
    let username = String::from(username);
    let fromdate = String::from(fromdate);
    let todate = String::from(todate);

    let article_allhistory_json = ArticleAllhistoryReq {
        group,
        token,
        username,
        count,
        fromdate,
        sortorder,
        todate,
    };
    client.post_request::<ArticleAllhistory, ArticleAllhistoryReq>(PATH, &article_allhistory_json)
}
