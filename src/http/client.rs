use std::env;

use reqwest::{
    blocking::Client as reqwest_client, blocking::Response,
    header::{HeaderMap, CONTENT_TYPE, ACCEPT_LANGUAGE, USER_AGENT, CONNECTION, ACCEPT_ENCODING, TE, AUTHORIZATION, ACCEPT},
};
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

#[cfg(feature = "401")]
use rand::Rng;

use crate::errors::*;
use crate::reqwest;

#[derive(Debug, Clone)]
struct Client {
    client: reqwest_client,
    base_url: String,
    x_talk_app_id: String,
}

impl Client {
    pub fn new(base_url: String, x_talk_app_id: String) -> Client {
        Client {
            client: reqwest_client::new(),
            base_url,
            x_talk_app_id,
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
        header.insert("X-Talk-App-ID", (&self.x_talk_app_id).parse()?);
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

pub trait SHClient: Clone {
    fn new() -> Self where Self: Sized;

    fn post_request<RT, JT>(&self, path: &str, json: &JT) -> Result<RT>
        where RT: DeserializeOwned, JT: Serialize + ?Sized;

    fn get_request<RT>(&self, path: &str, access_token: &str, parameters: Option<Vec<(&str, &str)>>) -> Result<RT>
        where RT: DeserializeOwned;
}

#[derive(Debug, Clone)]
pub struct SClient {
    client: Client,
}

impl SHClient for SClient {
    fn new() -> SClient {
        SClient {
            client: Client::new(
                base_url(),
                "jp.co.sonymusic.communication.sakurazaka 2.1".to_string(),
            ),
        }
    }

    fn post_request<RT, JT>(&self, path: &str, json: &JT) -> Result<RT>
        where RT: DeserializeOwned, JT: Serialize + ?Sized {
        self.client.post_request(path, json)
    }

    fn get_request<RT>(&self, path: &str, access_token: &str, parameters: Option<Vec<(&str, &str)>>) -> Result<RT>
        where RT: DeserializeOwned {
        self.client.get_request(path, access_token, parameters)
    }
}

fn base_url() -> String {
    env::var("BASE_S_URL")
        .ok()
        .unwrap_or_else(|| "https://api.s46.glastonr.net".to_string())
}
