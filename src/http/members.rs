use crate::{errors::*, http::client::SHNClient};
use serde::{Deserialize, Serialize};

const MEMBERS_PATH: &str = "/v2/members";
const PATH: &str = "/v2/groups";
const PATH2: &str = "/members";

#[derive(Serialize, Deserialize, Debug)]
pub struct Members {
    pub id: u32,
    pub name: String,
}

pub fn request_all<C: SHNClient>(client: C, access_token: &String) -> Result<Vec<Members>> {
    let access_token = String::from(access_token);

    client.get_request::<Vec<Members>>(MEMBERS_PATH, &access_token, None, false)
}

pub fn request<C: SHNClient>(
    client: C,
    access_token: &String,
    group_id: &u32,
) -> Result<Vec<Members>> {
    let path = format!("{}/{}{}", PATH, group_id, PATH2);
    let access_token = String::from(access_token);

    client.get_request::<Vec<Members>>(path.as_str(), &access_token, None, false)
}

#[cfg(test)]
mod tests {
    use super::{request, request_all};
    use crate::http::test_support::{Call, Client};
    use serde_json::json;

    #[test]
    fn request_all_returns_members_from_the_global_endpoint() {
        let client = Client::new(json!([
            {"id": 12, "name": "菅井 友香"},
            {"id": 84, "name": "藤吉 夏鈴"}
        ]));

        let members = request_all(client.clone(), &"access-token".to_owned()).unwrap();

        assert_eq!(members[0].id, 12);
        assert_eq!(members[0].name, "菅井 友香");
        assert_eq!(members[1].id, 84);
        assert_eq!(members[1].name, "藤吉 夏鈴");
        assert_eq!(
            client.calls(),
            vec![Call::Get {
                path: "/v2/members".to_owned(),
                access_token: "access-token".to_owned(),
                parameters: None,
                is_dynamic: false,
            }]
        );
    }

    #[test]
    fn request_returns_members_from_the_group_endpoint() {
        let client = Client::new(json!([{"id": 84, "name": "藤吉 夏鈴"}]));

        let members = request(client.clone(), &"access-token".to_owned(), &48).unwrap();

        assert_eq!(members[0].id, 84);
        assert_eq!(members[0].name, "藤吉 夏鈴");
        assert_eq!(
            client.calls(),
            vec![Call::Get {
                path: "/v2/groups/48/members".to_owned(),
                access_token: "access-token".to_owned(),
                parameters: None,
                is_dynamic: false,
            }]
        );
    }
}
