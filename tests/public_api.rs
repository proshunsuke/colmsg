use std::{io, process::Command};

use colmsg::{
    errors::{handle_error, Error, ErrorKind},
    http::{
        client::{AClient, HClient, MClient, NClient, SClient, SHNClient, YClient},
        timeline::{Timeline, TimelineMessages},
    },
};
use serde_json::{json, Value};

fn rejects_invalid_authorization<C: SHNClient>() {
    // Header validation happens before sending: no external request is made.
    let result = C::new().get_request::<Value>("/v2/groups", "invalid\r\ntoken", None, false);
    assert!(matches!(
        result.unwrap_err().kind(),
        ErrorKind::InvalidHeaderValue(_)
    ));
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
