use crate::{errors::*, http::client::SHNClient, http::timeline::TimelineMessages};
use serde::{Deserialize, Serialize};

const PATH: &str = "/v2/groups";
const PATH2: &str = "/past_messages";
const ORDER: &str = "asc";

#[derive(Serialize, Deserialize, Debug)]
pub struct PastMessages {
    pub messages: Vec<TimelineMessages>,
}

pub fn request<C: SHNClient>(client: C, access_token: &String, id: &u32) -> Result<PastMessages> {
    let path = format!("{}/{}{}", PATH, id, PATH2);
    let access_token = String::from(access_token);
    let parameters = vec![("order", ORDER)];

    client.get_request::<PastMessages>(path.as_str(), &access_token, Some(parameters), true)
}

#[cfg(test)]
mod tests {
    use super::request;
    use crate::http::test_support::{Call, Client};
    use serde_json::json;

    #[test]
    fn request_returns_past_messages_for_the_requested_member_in_ascending_order() {
        let client = Client::new(json!({"messages":[]}));

        let past = request(client.clone(), &"access-token".to_owned(), &42).unwrap();

        assert!(past.messages.is_empty());
        assert_eq!(
            client.calls(),
            vec![Call::Get {
                path: "/v2/groups/42/past_messages".to_owned(),
                access_token: "access-token".to_owned(),
                parameters: Some(vec![("order".to_owned(), "asc".to_owned())]),
                is_dynamic: true,
            }]
        );
    }
}
