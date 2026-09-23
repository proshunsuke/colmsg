use crate::{errors::*, http::client::SHNClient};
use serde::{Deserialize, Serialize};

const PATH: &str = "/v2/groups";
const PATH2: &str = "/timeline";
pub const DEFAULT_COUNT: usize = 100;
const ORDER: &str = "asc";

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelineComments {}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelineMessages {
    pub group_id: u32,
    pub id: u32,
    pub is_favorite: bool,
    pub is_silent: bool,
    pub member_id: Option<u32>,
    pub publish_type: Option<String>,
    pub published_at: String,
    pub state: String,
    pub text: Option<String>,
    #[serde(rename = "type")]
    pub messages_type: String,
    // link 型でのみ出現する追加パラメータ
    pub link_params: Option<LinkParams>,
    pub updated_at: String,
    pub file: Option<String>,
    pub thumbnail: Option<String>,
    pub thumbnail_height: Option<u32>,
    pub thumbnail_width: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkParams {
    pub url: String,
    pub method: String,
    #[serde(rename = "sendid")]
    pub send_id: Option<u32>,
    // 仕様が不定のため汎用型で受ける
    pub parameters: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelineLetters {
    pub client_token: String,
    pub created_at: String,
    pub file: String,
    pub group_id: u32,
    pub id: u32,
    pub is_favorite: bool,
    pub member_id: Option<u32>,
    pub opened_at: Option<String>,
    pub text: String,
    pub thumbnail: String,
    pub thumbnail_height: u32,
    pub thumbnail_width: u32,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub comments: Vec<TimelineComments>,
    pub letters: Vec<TimelineLetters>,
    pub messages: Vec<TimelineMessages>,
    pub queried_at: String,
}

pub fn request<C: SHNClient>(
    client: C,
    access_token: &String,
    id: &u32,
    fromdate: &String,
    count: &String,
) -> Result<Timeline> {
    let path = format!("{}/{}{}", PATH, id, PATH2);
    let access_token = String::from(access_token);
    let parameters = vec![
        ("created_from", "2000-01-01T00:00:00Z"),
        ("updated_from", fromdate),
        ("count", count),
        ("order", ORDER),
    ];

    client.get_request::<Timeline>(path.as_str(), &access_token, Some(parameters), true)
}

#[cfg(test)]
mod tests {
    use super::request;
    use crate::http::test_support::{Call, Client};
    use serde_json::json;

    #[test]
    fn request_returns_timeline_and_uses_requested_member_and_pagination() {
        let client = Client::new(json!({"comments":[], "letters":[], "messages":[],
            "queried_at":"2026-09-23T01:02:03Z"}));

        let timeline = request(
            client.clone(),
            &"access-token".to_owned(),
            &42,
            &"2026-09-20T12:34:56Z".to_owned(),
            &"250".to_owned(),
        )
        .unwrap();

        assert!(timeline.messages.is_empty());
        assert_eq!(timeline.queried_at, "2026-09-23T01:02:03Z");
        assert_eq!(
            client.calls(),
            vec![Call::Get {
                path: "/v2/groups/42/timeline".to_owned(),
                access_token: "access-token".to_owned(),
                parameters: Some(vec![
                    ("created_from".to_owned(), "2000-01-01T00:00:00Z".to_owned()),
                    ("updated_from".to_owned(), "2026-09-20T12:34:56Z".to_owned()),
                    ("count".to_owned(), "250".to_owned()),
                    ("order".to_owned(), "asc".to_owned()),
                ]),
                is_dynamic: true,
            }]
        );
    }
}
