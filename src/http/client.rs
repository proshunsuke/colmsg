use std::env;

use reqwest::blocking::Client as reqwest_client;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, CONTENT_TYPE, ACCEPT_LANGUAGE, USER_AGENT, CONNECTION, ACCEPT_ENCODING, TE, AUTHORIZATION, ACCEPT};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

#[cfg(feature = "401")]
use rand::Rng;

use crate::errors::*;
use crate::reqwest;

pub struct Client {
    client: reqwest_client,
    base_url: String,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest_client::new(),
            base_url: base_url(),
        }
    }

    pub fn post_request<RT, JT>(&self, path: &str, json: &JT) -> Result<RT>
        where RT: DeserializeOwned, JT: Serialize + ?Sized {
        let url = Url::parse(&self.base_url)?.join(&path)?;
        let request_url = url.as_ref().to_string();
        let response: Response = self.client
            .post(url)
            .headers(self.insert_headers(HeaderMap::new())?)
            .json(json)
            .send()?
            .error_for_status()?;
        self.handle_response(response, &request_url)
    }

    pub fn get_request<RT>(&self, path: &str, access_token: &str, parameters: Option<Vec<(&str, &str)>>) -> Result<RT>
        where RT: DeserializeOwned {
        let mut header = self.insert_headers(HeaderMap::new())?;
        header = self.insert_optional_headers(header, access_token)?;

        let iter = match parameters {
            Some(v) => v,
            None => vec![]
        };

        let url = Url::parse(&self.base_url)?.join(&path)?;
        let url = Url::parse_with_params(url.as_str(), &iter)?;
        let request_url = url.as_ref().to_string();
        let response: Response = self.client
            .get(url)
            .headers(header)
            .send()?
            .error_for_status()?;
        self.handle_response(response, &request_url)
    }

    fn handle_response<RT>(&self, response: Response, request_url: &String) -> Result<RT>
        where RT: DeserializeOwned {
        let body = response.text()?;
        let result = serde_json::from_str::<RT>(&body);
        match result {
            Ok(t) => Ok(t),
            Err(e) => {
                let error_message = format!(
                    "error: {}, request url: {}, response body: {}", e.to_string(), request_url, &body
                );
                Err(error_message.into())
            }
        }
    }

    fn insert_headers(&self, mut header: HeaderMap) -> Result<HeaderMap> {
        header.insert(ACCEPT, "application/json".parse()?);
        header.insert(CONTENT_TYPE, "application/json".parse()?);
        header.insert("X-Talk-App-ID", "jp.co.sonymusic.communication.keyakizaka 2.0".parse()?);
        header.insert(ACCEPT_LANGUAGE, "ja-JP".parse()?);
        header.insert(USER_AGENT, "Dalvik/2.1.0 (Linux; U; Android 6.0; Samsung Galaxy S7 for keyaki messages Build/MRA58K)".parse()?);
        header.insert(CONNECTION, "Keep-Alive".parse()?);
        header.insert(ACCEPT_ENCODING, "gzip".parse()?);
        header.insert(TE, "gzip, deflate; q=0.5".parse()?);
        Ok(header)
    }

    fn insert_optional_headers(&self, mut header: HeaderMap, access_token: &str) -> Result<HeaderMap> {
        let authorization = format!("Bearer {}", access_token);
        header.insert(AUTHORIZATION, authorization.parse()?);
        header = self.insert_401_header(header)?;
        Ok(header)
    }

    #[cfg(feature = "401")]
    fn insert_401_header(&self, mut header: HeaderMap) -> Result<HeaderMap> {
        if rand::thread_rng().gen_range(0, 6) == 0 {
            header.insert("Prefer", "code=401".parse()?);
        };
        Ok(header)
    }

    #[cfg(not(feature = "401"))]
    fn insert_401_header(&self, header: HeaderMap) -> Result<HeaderMap> { Ok(header) }
}

fn base_url() -> String {
    env::var("BASE_URL")
        .ok()
        .unwrap_or_else(|| "https://api.kh.glastonr.net".to_string())
}
