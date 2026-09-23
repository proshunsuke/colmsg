use crate::{errors::*, http::client::SHNClient};
use serde::{Deserialize, Serialize};

const PATH: &str = "/v2/tags";

#[derive(Serialize, Deserialize, Debug)]
pub struct TagsMeta {
    pub color: Option<String>, // memo: khは必須だがには無い
    pub dimension: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tags {
    pub meta: Option<TagsMeta>, // memo: khは必須だがには無い場合がある
    pub name: String,
    pub priority: u32,
    pub updated_at: String,
    pub uuid: String,
}

pub fn request<C: SHNClient>(client: C, access_token: &String) -> Result<Vec<Tags>> {
    let access_token = String::from(access_token);

    client.get_request::<Vec<Tags>>(PATH, &access_token, None, false)
}

#[cfg(test)]
mod tests {
    use super::request;
    use crate::http::test_support::{Call, Client};
    use serde_json::json;

    #[test]
    fn request_returns_tags_and_preserves_optional_metadata() {
        let client = Client::new(json!([
            {"uuid":"generation", "name":"一期生", "priority":0,
             "updated_at":"2026-09-23T01:02:03Z", "meta":{"color":"#ffffff"}},
            {"uuid":"group", "name":"グループ", "priority":1,
             "updated_at":"2026-09-23T01:02:03Z", "meta":null}
        ]));

        let tags = request(client.clone(), &"access-token".to_owned()).unwrap();

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "一期生");
        assert_eq!(tags[0].meta.as_ref().unwrap().dimension, None);
        assert!(tags[1].meta.is_none());
        assert_eq!(
            client.calls(),
            vec![Call::Get {
                path: "/v2/tags".to_owned(),
                access_token: "access-token".to_owned(),
                parameters: None,
                is_dynamic: false,
            }]
        );
    }
}
