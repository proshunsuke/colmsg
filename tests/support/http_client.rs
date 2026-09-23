use std::sync::{Arc, Mutex};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{errors::Result, http::client::SHNClient};

#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    Get {
        path: String,
        access_token: String,
        parameters: Option<Vec<(String, String)>>,
        is_dynamic: bool,
    },
    Post {
        path: String,
        body: Value,
        is_dynamic: bool,
    },
}

#[derive(Clone)]
pub struct Client {
    response: Value,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Client {
    pub fn new(response: Value) -> Self {
        Self {
            response,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }
}

impl SHNClient for Client {
    fn new() -> Self {
        unreachable!("test client needs a response")
    }

    fn post_request<RT, JT>(&self, path: &str, json: &JT, is_dynamic: bool) -> Result<RT>
    where
        RT: DeserializeOwned,
        JT: Serialize + ?Sized,
    {
        self.calls.lock().unwrap().push(Call::Post {
            path: path.to_owned(),
            body: serde_json::to_value(json).unwrap(),
            is_dynamic,
        });
        Ok(serde_json::from_value(self.response.clone()).unwrap())
    }

    fn get_request<RT>(
        &self,
        path: &str,
        access_token: &str,
        parameters: Option<Vec<(&str, &str)>>,
        is_dynamic: bool,
    ) -> Result<RT>
    where
        RT: DeserializeOwned,
    {
        self.calls.lock().unwrap().push(Call::Get {
            path: path.to_owned(),
            access_token: access_token.to_owned(),
            parameters: parameters.map(|items| {
                items
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value.to_owned()))
                    .collect()
            }),
            is_dynamic,
        });
        Ok(serde_json::from_value(self.response.clone()).unwrap())
    }
}
