use std::env;

use reqwest::blocking::Client as reqwest_client;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, CONTENT_TYPE, ACCEPT_LANGUAGE, USER_AGENT, CONNECTION, ACCEPT_ENCODING};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::reqwest;

pub struct Client {
    client: reqwest_client,
    base_url: String,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest_client::new(),
            base_url: base_url()
        }
    }

    pub fn post_request<RT, JT>(&self, path: &str, json: &JT) -> Result<RT, reqwest::Error>
        where RT: DeserializeOwned, JT: Serialize + ?Sized {
        let url = Url::parse(&self.base_url).unwrap().join(&path).unwrap();
        self.client
            .post(url)
            .headers(self.header())
            .json(json)
            .send()?
            .json::<RT>()
    }

    pub fn get_request(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.client
            .get(url)
            .headers(self.header())
            .send()
    }

    fn header(&self) -> HeaderMap {
        let mut subscribe_list_headers = HeaderMap::new();
        subscribe_list_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        subscribe_list_headers.insert("X-API-Version", "1.7.0".parse().unwrap());
        subscribe_list_headers.insert(ACCEPT_LANGUAGE, "ja-JP".parse().unwrap());
        subscribe_list_headers.insert(USER_AGENT, "Dalvik/2.1.0 (Linux; U; Android 6.0; Samsung Galaxy S7 for keyaki messages Build/MRA58K)".parse().unwrap());
        subscribe_list_headers.insert(CONNECTION, "Keep-Alive".parse().unwrap());
        subscribe_list_headers.insert(ACCEPT_ENCODING, "gzip".parse().unwrap());
        subscribe_list_headers
    }
}

fn base_url() -> String {
    env::var("BASE_URL")
        .ok()
        .unwrap_or_else(|| "https://client-k.hot.sonydna.com".to_string())
}
