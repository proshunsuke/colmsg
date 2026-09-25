use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use assert_cmd::Command;
use mockito::{Matcher, Mock, Server, ServerGuard};
use serde_json::{json, Value};
use tempdir::TempDir;

pub const INITIAL_DATE: &str = "2000-01-01T09:00:00Z";
pub const DATE: &str = "2026-09-23T01:02:03Z";
pub const TEXT: &str = "テスト用メッセージ☀\n二行目";
pub const MEDIA: &[u8] = b"\x00\xff\x01test media\r\n\x80";

#[derive(Clone, Copy)]
pub struct Service {
    pub group: &'static str,
    pub token_flag: &'static str,
    pub base_env: &'static str,
    pub app_id: &'static str,
}

pub const SERVICES: [Service; 6] = [
    Service {
        group: "sakurazaka",
        token_flag: "--s_refresh_token",
        base_env: "S_BASE_URL",
        app_id: "jp.co.sonymusic.communication.sakurazaka 2.4",
    },
    Service {
        group: "hinatazaka",
        token_flag: "--h_refresh_token",
        base_env: "H_BASE_URL",
        app_id: "jp.co.sonymusic.communication.keyakizaka 2.4",
    },
    Service {
        group: "nogizaka",
        token_flag: "--n_refresh_token",
        base_env: "N_BASE_URL",
        app_id: "jp.co.sonymusic.communication.nogizaka 2.4",
    },
    Service {
        group: "asukasaito",
        token_flag: "--a_refresh_token",
        base_env: "A_BASE_URL",
        app_id: "jp.co.sonymusic.communication.asukasaito 2.4",
    },
    Service {
        group: "maishiraishi",
        token_flag: "--m_refresh_token",
        base_env: "M_BASE_URL",
        app_id: "jp.co.sonymusicsolutions.maishiraishi 2.4",
    },
    Service {
        group: "yodel",
        token_flag: "--y_refresh_token",
        base_env: "Y_BASE_URL",
        app_id: "jp.co.sonymusic.communication.yodel 2.4",
    },
];

pub struct Scenario {
    pub root: TempDir,
    pub server: ServerGuard,
    pub service: Service,
    global_members_mock: Option<Mock>,
    group_members_mock: Option<Mock>,
}

impl Scenario {
    pub fn new(service: Service) -> Self {
        let root = TempDir::new("colmsg-test").unwrap();
        fs::create_dir_all(root.path().join("home/Downloads")).unwrap();
        fs::create_dir_all(root.path().join("config/colmsg")).unwrap();
        fs::write(root.path().join("config/colmsg/config"), "").unwrap();
        let mut scenario = Scenario {
            root,
            server: Server::new(),
            service,
            global_members_mock: None,
            group_members_mock: None,
        };
        scenario.global_members_mock = Some(scenario.get("/v2/members", json!([])));
        scenario.group_members_mock =
            Some(scenario.get_at_most_once("/v2/groups/1/members", json!([])));
        scenario
    }

    pub fn output(&self) -> PathBuf {
        self.root.path().join("output")
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.path().join("config/colmsg/config")
    }

    pub fn token_file(&self) -> PathBuf {
        self.root.path().join("config/colmsg").join(
            self.service
                .token_flag
                .trim_start_matches("--")
                .replace("refresh_token", "access_token"),
        )
    }

    pub fn command(&self) -> Command {
        let mut command = Command::cargo_bin("colmsg").unwrap();
        command.timeout(Duration::from_secs(10));
        command.current_dir(self.root.path());
        // Only the child sees these values: parallel tests never mutate process-wide state.
        command.env("HOME", self.root.path().join("home"));
        command.env("USERPROFILE", self.root.path().join("home"));
        command.env("XDG_CONFIG_HOME", self.root.path().join("config"));
        command.env("APPDATA", self.root.path().join("config"));
        command.env("COLMSG_CONFIG_PATH", self.config_file());
        command.env("COLMSG_CONFIG_DIR", self.root.path().join("config/colmsg"));
        command.env("NO_PROXY", "*");
        command.env("no_proxy", "*");
        for key in &[
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
        ] {
            command.env_remove(key);
        }
        // Even an incorrectly selected service must never fall back to a real API.
        for service in &SERVICES {
            command.env(service.base_env, self.server.url());
        }
        command
    }

    pub fn download(&self) -> Command {
        let mut command = self.command();
        command.args([self.service.token_flag, "test-refresh-token", "--dir"]);
        command.arg(self.output());
        command
    }

    pub fn auth(&mut self) -> Mock {
        self.server.mock("POST", "/v2/update_token")
            .match_header("x-talk-app-id", self.service.app_id)
            .match_header("content-type", "application/json")
            .match_header("authorization", Matcher::Missing)
            .match_body(Matcher::Json(json!({"refresh_token": "test-refresh-token"})))
            .with_header("content-type", "application/json")
            .with_body(json!({"access_token":format!("test-access-token-{}", self.service.group), "expires_in":3600, "refresh_token":"test-refresh-token"}).to_string())
            .expect(1).create()
    }

    pub fn get(&mut self, path: &str, body: Value) -> Mock {
        self.get_with_expectation(path, body, true)
    }

    fn get_at_most_once(&mut self, path: &str, body: Value) -> Mock {
        self.get_with_expectation(path, body, false)
    }

    fn get_with_expectation(&mut self, path: &str, body: Value, required: bool) -> Mock {
        let path = if path.contains('?') {
            path.to_string()
        } else {
            format!("{}?", path)
        };
        let mock = self
            .server
            .mock("GET", path.as_str())
            .match_header("x-talk-app-id", self.service.app_id)
            .match_header(
                "authorization",
                format!("Bearer test-access-token-{}", self.service.group).as_str(),
            )
            .match_header("accept", "application/json")
            .with_header("content-type", "application/json")
            .with_body(body.to_string());
        if required {
            mock.expect(1).create()
        } else {
            mock.expect_at_most(1).create()
        }
    }

    pub fn set_global_members(&mut self, body: Value) {
        if let Some(mock) = self.global_members_mock.take() {
            mock.remove();
        }
        self.global_members_mock = Some(self.get("/v2/members", body));
    }

    #[allow(dead_code)]
    pub fn fail_global_members(&mut self, status: usize) -> Mock {
        if let Some(mock) = self.global_members_mock.take() {
            mock.remove();
        }
        self.server
            .mock("GET", "/v2/members?")
            .match_header("x-talk-app-id", self.service.app_id)
            .match_header(
                "authorization",
                format!("Bearer test-access-token-{}", self.service.group).as_str(),
            )
            .match_header("accept", "application/json")
            .with_status(status)
            .expect(1)
            .create()
    }

    #[allow(dead_code)]
    pub fn fail_group_members(&mut self, status: usize) -> Mock {
        if let Some(mock) = self.group_members_mock.take() {
            mock.remove();
        }
        self.server
            .mock("GET", "/v2/groups/1/members?")
            .match_header("x-talk-app-id", self.service.app_id)
            .match_header(
                "authorization",
                format!("Bearer test-access-token-{}", self.service.group).as_str(),
            )
            .match_header("accept", "application/json")
            .with_status(status)
            .expect(1)
            .create()
    }

    pub fn set_group_members(&mut self, body: Value) {
        if let Some(mock) = self.group_members_mock.take() {
            mock.remove();
        }
        self.group_members_mock = Some(self.get("/v2/groups/1/members", body));
    }

    pub fn assert_member_mocks(&self) {
        self.global_members_mock.as_ref().unwrap().assert();
        self.group_members_mock.as_ref().unwrap().assert();
    }

    pub fn catalog(&mut self) -> Vec<Mock> {
        vec![
            self.get(
                "/v2/groups",
                json!([group(1, "テスト メンバー", true, &["generation"])]),
            ),
            self.get(
                "/v2/tags",
                json!([tag("generation", "一期生", json!({"color":"#ffffff"}))]),
            ),
        ]
    }

    pub fn past(&mut self, messages: Vec<Value>) -> Mock {
        self.past_for(1, messages)
    }

    pub fn past_for(&mut self, group_id: u32, messages: Vec<Value>) -> Mock {
        self.get(
            &format!("/v2/groups/{}/past_messages?order=asc", group_id),
            json!({"messages":messages}),
        )
    }

    pub fn timeline(&mut self, from: &str, count: usize, messages: Vec<Value>) -> Mock {
        self.timeline_for(1, from, count, messages)
    }

    pub fn timeline_for(
        &mut self,
        group_id: u32,
        from: &str,
        count: usize,
        messages: Vec<Value>,
    ) -> Mock {
        self.server
            .mock("GET", format!("/v2/groups/{}/timeline", group_id).as_str())
            .match_header("x-talk-app-id", self.service.app_id)
            .match_header(
                "authorization",
                format!("Bearer test-access-token-{}", self.service.group).as_str(),
            )
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("created_from".into(), "2000-01-01T00:00:00Z".into()),
                Matcher::UrlEncoded("updated_from".into(), from.into()),
                Matcher::UrlEncoded("count".into(), count.to_string()),
                Matcher::UrlEncoded("order".into(), "asc".into()),
            ]))
            .with_header("content-type", "application/json")
            .with_body(timeline(messages).to_string())
            .expect(1)
            .create()
    }

    pub fn member_dir(&self) -> PathBuf {
        self.output().join("一期生/テストメンバー")
    }

    pub fn media(&mut self, path: &str) -> Mock {
        self.server
            .mock("GET", path)
            .match_header("authorization", Matcher::Missing)
            .with_body(MEDIA)
            .expect(1)
            .create()
    }
}

pub fn group(id: u32, name: &str, subscribed: bool, tags: &[&str]) -> Value {
    json!({"id":id, "is_letter_destination":false, "name":name,
        "priority":0, "state":"active", "tags":tags, "thumbnail":"", "updated_at":DATE,
        "subscription": if subscribed { json!({"auto_renewing":true,"start_at":DATE,"type":"paid"}) } else { Value::Null }})
}

pub fn tag(uuid: &str, name: &str, meta: Value) -> Value {
    json!({"uuid":uuid,"name":name,"priority":0,"updated_at":DATE,"meta":meta})
}

pub fn message(id: u32, kind: &str, base: &str) -> Value {
    let mut value = json!({"group_id":1, "id":id, "is_favorite":false, "is_silent":false,
        "published_at":DATE, "updated_at":DATE, "state":"published", "type":kind});
    if ["text", "picture", "link"].contains(&kind) {
        value["text"] = json!(TEXT);
    }
    if ["picture", "video", "voice"].contains(&kind) {
        value["file"] = json!(format!("{}/media/{}", base, id));
    }
    if kind == "link" {
        value["link_params"] = json!({"url":"https://example.invalid/", "method":"GET", "sendid":1,"parameters":[{"key":"value"}, 1, null]});
    }
    value
}

pub fn timeline(messages: Vec<Value>) -> Value {
    json!({"comments":[],"letters":[],"messages":messages,"queried_at":DATE})
}

pub fn files(dir: &Path) -> Vec<PathBuf> {
    let mut result = walkdir::WalkDir::new(dir)
        .into_iter()
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().strip_prefix(dir).unwrap().to_path_buf())
        .collect::<Vec<_>>();
    result.sort();
    result
}
