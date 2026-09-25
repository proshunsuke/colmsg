use std::{
    env,
    ffi::OsString,
    io,
    process::Command,
    sync::{Mutex, MutexGuard},
};

use colmsg::{
    controller::{Controller, Service},
    errors::{handle_error, Error},
    http::{
        client::{AClient, HClient, MClient, NClient, SClient, SHNClient, YClient},
        timeline::{Timeline, TimelineMessages},
    },
    Config, Kind,
};
use serde_json::{json, Value};

static BASE_URL_ENV_LOCK: Mutex<()> = Mutex::new(());

fn rejects_invalid_authorization<C: SHNClient>() {
    // Header validation happens before sending: no external request is made.
    let result = C::new().get_request::<Value>("/v2/groups", "invalid\r\ntoken", None, false);
    assert!(matches!(result.unwrap_err(), Error::InvalidHeaderValue(_)));
}

#[test]
fn all_public_clients_reject_invalid_authorization_without_network() {
    rejects_invalid_authorization::<SClient>();
    rejects_invalid_authorization::<HClient>();
    rejects_invalid_authorization::<NClient>();
    rejects_invalid_authorization::<AClient>();
    rejects_invalid_authorization::<MClient>();
    rejects_invalid_authorization::<YClient>();
}

#[test]
fn timeline_accepts_letters_and_optional_link_parameters() {
    let value = json!({
        "queried_at":"2026-09-23T01:02:03Z", "comments":[{}],
        "letters":[{"client_token":"test", "created_at":"2026-09-23T01:02:03Z", "file":"https://example.invalid/letter", "group_id":1,
            "id":2,"is_favorite":false,"member_id":null,"opened_at":null,"text":"手紙", "thumbnail":"", "thumbnail_height":0,"thumbnail_width":0,"updated_at":"2026-09-23T01:02:03Z"}],
        "messages":[{"group_id":1,"id":3,"is_favorite":false,"is_silent":true,"member_id":null,"published_at":"2026-09-23T01:02:03Z",
            "updated_at":"2026-09-23T01:02:03Z","state":"published","type":"link","text":"リンク",
            "link_params":{"url":"https://example.invalid/","method":"POST","parameters":[null,1,"text",{"key":true}]}}]
    });
    let timeline: Timeline = serde_json::from_value(value).unwrap();
    assert_eq!(timeline.letters[0].text, "手紙");
    assert_eq!(timeline.messages[0].messages_type, "link");
    let params = timeline.messages[0].link_params.as_ref().unwrap();
    assert_eq!(params.send_id, None);
    assert_eq!(params.method, "POST");
    assert_eq!(
        params.parameters,
        vec![json!(null), json!(1), json!("text"), json!({"key":true})]
    );
    let encoded = serde_json::to_value(&timeline).unwrap();
    assert_eq!(encoded["messages"][0]["type"], "link");
    assert_eq!(encoded["letters"][0]["id"], 2);
}

#[test]
fn required_fields_and_wrong_types_are_not_silently_accepted() {
    for value in [json!({}), json!({"group_id":"1","id":1,"is_favorite":false,"is_silent":false,
        "published_at":"2026-09-23T01:02:03Z","updated_at":"2026-09-23T01:02:03Z","state":"published","type":"text"})].iter() {
        assert!(serde_json::from_value::<TimelineMessages>(value.clone()).is_err());
    }
}

struct BaseUrlEnvGuard {
    previous: Vec<(&'static str, Option<OsString>)>,
    _lock: MutexGuard<'static, ()>,
}

impl BaseUrlEnvGuard {
    fn set_only(name: &'static str, value: &str) -> Self {
        const BASE_URLS: [&str; 3] = ["S_BASE_URL", "H_BASE_URL", "N_BASE_URL"];
        let lock = BASE_URL_ENV_LOCK.lock().unwrap();
        let previous = BASE_URLS
            .iter()
            .map(|name| (*name, env::var_os(name)))
            .collect();

        for base_url in BASE_URLS {
            if base_url == name {
                env::set_var(base_url, value);
            } else {
                env::remove_var(base_url);
            }
        }

        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for BaseUrlEnvGuard {
    fn drop(&mut self) {
        for (name, value) in &self.previous {
            if let Some(value) = value {
                env::set_var(name, value);
            } else {
                env::remove_var(name);
            }
        }
    }
}

fn dynamic_mock_request_sends_prefer_header<C: SHNClient>(base_url_env: &'static str) {
    let mut server = mockito::Server::new();
    let _base_url_guard = BaseUrlEnvGuard::set_only(base_url_env, &server.url());
    let mock = server
        .mock("GET", "/v2/members?")
        .match_header("prefer", "dynamic=true")
        .with_header("content-type", "application/json")
        .with_body("[]")
        .expect(1)
        .create();

    let members: Vec<Value> = C::new()
        .get_request("/v2/members", "access-token", None, true)
        .unwrap();

    assert!(members.is_empty());
    mock.assert();
}

#[test]
fn dynamic_mock_requests_send_prefer_for_each_openapi_base_url() {
    dynamic_mock_request_sends_prefer_header::<SClient>("S_BASE_URL");
    dynamic_mock_request_sends_prefer_header::<HClient>("H_BASE_URL");
    dynamic_mock_request_sends_prefer_header::<NClient>("N_BASE_URL");
}

#[test]
fn controller_public_methods_keep_the_legacy_path_and_reject_zero_jobs() {
    const APP_ID: &str = "jp.co.sonymusic.communication.sakurazaka 2.4";
    let mut server = mockito::Server::new();
    let _base_url_guard = BaseUrlEnvGuard::set_only("S_BASE_URL", &server.url());
    let requests = ["/v2/members?", "/v2/groups?", "/v2/tags?"]
        .iter()
        .map(|path| {
            server
                .mock("GET", *path)
                .match_header("x-talk-app-id", APP_ID)
                .match_header("authorization", "Bearer access-token")
                .match_header("accept", "application/json")
                .with_header("content-type", "application/json")
                .with_body("[]")
                .expect(1)
                .create()
        })
        .collect::<Vec<_>>();
    let root = tempdir::TempDir::new("colmsg-controller-public-api").unwrap();
    let config = Config {
        name: vec![],
        from: None,
        kind: vec![Kind::Text],
        dir: root.path().to_path_buf(),
        client: SClient::new(),
        access_token: "access-token".to_owned(),
    };
    let controller = Controller::new(&config);

    controller.run().unwrap();
    for request in requests {
        request.assert();
    }

    let (progress, _) = std::sync::mpsc::channel();
    let error = controller
        .run_with_progress(0, Service::Sakurazaka, &progress)
        .unwrap_err();
    assert_eq!(error.to_string(), "jobs must be greater than zero");
}

#[test]
fn public_error_handler_handles_broken_pipe_and_reports_other_errors() {
    const MODE: &str = "COLMSG_TEST_ERROR_HANDLER_MODE";
    if let Ok(mode) = std::env::var(MODE) {
        let kind = if mode == "pipe" {
            io::ErrorKind::BrokenPipe
        } else {
            io::ErrorKind::PermissionDenied
        };
        handle_error(&Error::from(io::Error::new(kind, "test-output-error")));
        println!("handler returned");
        return;
    }
    for mode in &["pipe", "permission"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "public_error_handler_handles_broken_pipe_and_reports_other_errors",
                "--nocapture",
            ])
            .env(MODE, mode)
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if *mode == "pipe" {
            assert!(!stdout.contains("handler returned"));
            assert!(!stderr.contains("test-output-error"));
        } else {
            assert!(stdout.contains("handler returned"));
            assert!(stderr.contains("test-output-error"));
        }
    }
}
