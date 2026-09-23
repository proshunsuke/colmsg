use crate::{errors::*, http::client::SHNClient};
use serde::{Deserialize, Serialize};

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
    pub refresh_token: String,
}

pub fn request<C: SHNClient>(client: C, refresh_token: &String) -> Result<UpdateToken> {
    let refresh_token = String::from(refresh_token);

    let update_token_json = UpdateTokenReq { refresh_token };
    client.post_request::<UpdateToken, UpdateTokenReq>(PATH, &update_token_json, true)
}

#[cfg(test)]
mod tests {
    use super::request;
    use crate::http::test_support::{Call, Client};
    use serde_json::json;

    #[test]
    fn request_sends_the_refresh_token_and_returns_rotated_tokens() {
        let client = Client::new(json!({"access_token":"new-access", "expires_in":3600,
            "refresh_token":"new-refresh"}));

        let tokens = request(client.clone(), &"old-refresh".to_owned()).unwrap();

        assert_eq!(tokens.access_token, "new-access");
        assert_eq!(tokens.refresh_token, "new-refresh");
        assert_eq!(tokens.expires_in, 3600);
        assert_eq!(
            client.calls(),
            vec![Call::Post {
                path: "/v2/update_token".to_owned(),
                body: json!({"refresh_token":"old-refresh"}),
                is_dynamic: true,
            }]
        );
    }
}
