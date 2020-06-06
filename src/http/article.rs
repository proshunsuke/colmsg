use serde::{Deserialize, Serialize};
use crate::http;
use crate::errors::*;

const PATH: &str = "/article";

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleResult {
    pub disable: u32,
    pub phoneimage: Option<String>,
    pub thumbnail: String,
    pub url: String,
    pub silent: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Article {
    pub result: ArticleResult,
    pub status: String,
    pub statuscd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleReq {
    pub article: String,
    pub token: String,
    pub username: String,
}

pub fn request(article: &String, token: &String, username: &String) -> Result<Article> {
    let client = http::Client::new();
    let article = String::from(article);
    let token = String::from(token);
    let username = String::from(username);

    let article_json = ArticleReq { article, token, username };
    client.post_request::<Article, ArticleReq>(PATH, &article_json)
}
