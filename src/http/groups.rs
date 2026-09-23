use crate::{errors::*, http::client::SHNClient};
use serde::{Deserialize, Serialize};

const PATH: &str = "/v2/groups";

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupsSubscription {
    pub auto_renewing: bool,
    pub end_at: Option<String>,
    pub start_at: String,
    #[serde(rename = "type")]
    pub subscription_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Groups {
    pub id: u32,
    pub is_letter_destination: bool,
    pub name: String,
    pub phone_image: Option<String>,
    pub priority: u32,
    pub state: String,
    pub tags: Vec<String>,
    pub thumbnail: String,
    pub trial_days: Option<u32>,
    pub updated_at: String,
    pub subscription: Option<GroupsSubscription>,
}

pub fn request<C: SHNClient>(client: C, access_token: &String) -> Result<Vec<Groups>> {
    let access_token = String::from(access_token);

    client.get_request::<Vec<Groups>>(PATH, &access_token, None, false)
}

#[cfg(test)]
mod tests {
    use super::request;
    use crate::http::test_support::{Call, Client};
    use serde_json::json;

    #[test]
    fn request_returns_groups_using_the_groups_endpoint_without_dynamic_data() {
        let client = Client::new(json!([{
            "id": 12, "name": "テストメンバー", "is_letter_destination": false,
            "priority": 1, "state": "active", "tags": ["generation"],
            "thumbnail": "thumbnail", "updated_at": "2026-09-23T01:02:03Z",
            "subscription": null
        }]));

        let groups = request(client.clone(), &"access-token".to_owned()).unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id, 12);
        assert_eq!(groups[0].name, "テストメンバー");
        assert!(groups[0].subscription.is_none());
        assert_eq!(
            client.calls(),
            vec![Call::Get {
                path: "/v2/groups".to_owned(),
                access_token: "access-token".to_owned(),
                parameters: None,
                is_dynamic: false,
            }]
        );
    }
}
