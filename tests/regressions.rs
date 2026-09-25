mod common;

use common::*;
use serde_json::json;
use std::fs;

#[test]
fn poster_names_prefer_group_members_then_global_members_then_unknown() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let groups = s.get(
        "/v2/groups",
        json!([
            group(1, "テスト メンバー", true, &[]),
            group(2, "別メンバー", true, &[])
        ]),
    );
    let tags = s.get("/v2/tags", json!([]));
    s.set_global_members(json!([
        {"id": 1, "name": "全体側の名前"},
        {"id": 2, "name": "全体投稿者"},
        {"id": 4, "name": "全体側の補完名"},
        {"id": 6, "name": "空のグループ一覧からの補完名"},
        {"id": 7, "name": " \t"}
    ]));
    s.set_group_members(json!([
        {"id": 1, "name": "グループ投稿者"},
        {"id": 4, "name": ""},
        {"id": 7, "name": "  \t "}
    ]));
    let group_two_members = s.get("/v2/groups/2/members", json!([]));
    let past_one = s.past(vec![]);
    let past_two = s.past_for(2, vec![]);
    let base = s.server.url();
    let mut group_member = message(1, "text", &base);
    group_member["member_id"] = json!(1);
    let mut global_member = message(2, "text", &base);
    global_member["member_id"] = json!(2);
    let mut unknown_member = message(3, "text", &base);
    unknown_member["member_id"] = json!(3);
    let mut empty_group_name = message(4, "text", &base);
    empty_group_name["member_id"] = json!(4);
    let missing_member_id = message(5, "text", &base);
    let mut global_member_of_empty_group = message(6, "text", &base);
    global_member_of_empty_group["group_id"] = json!(2);
    global_member_of_empty_group["member_id"] = json!(6);
    let mut blank_member_names = message(7, "text", &base);
    blank_member_names["member_id"] = json!(7);
    let page_one = s.timeline(
        INITIAL_DATE,
        100,
        vec![
            group_member,
            global_member,
            unknown_member,
            empty_group_name,
            missing_member_id,
            blank_member_names,
        ],
    );
    let page_two = s.timeline_for(2, INITIAL_DATE, 100, vec![global_member_of_empty_group]);

    s.download().assert().success();

    assert_eq!(
        files(&s.output()),
        vec![
            "テストメンバー/1_0_20260923010203_グループ投稿者.txt",
            "テストメンバー/2_0_20260923010203_全体投稿者.txt",
            "テストメンバー/3_0_20260923010203_unknown.txt",
            "テストメンバー/4_0_20260923010203_全体側の補完名.txt",
            "テストメンバー/5_0_20260923010203_unknown.txt",
            "テストメンバー/7_0_20260923010203_unknown.txt",
            "別メンバー/6_0_20260923010203_空のグループ一覧からの補完名.txt",
        ]
        .into_iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>()
    );
    auth.assert();
    groups.assert();
    tags.assert();
    s.assert_member_mocks();
    group_two_members.assert();
    past_one.assert();
    past_two.assert();
    page_one.assert();
    page_two.assert();
}

#[test]
fn invalid_past_message_stops_before_timeline_request() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let mut invalid_message = message(1, "unsupported", &s.server.url());
    invalid_message["member_id"] = json!(1);
    let past = s.past(vec![invalid_message]);

    let output = s.download().output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown type: unsupported"));
    assert!(files(&s.output()).is_empty());
    auth.assert();
    past.assert();
    s.assert_member_mocks();
    for mock in catalog {
        mock.assert();
    }
}

#[test]
fn media_http_errors_never_create_successful_downloads() {
    for status in [403, 404, 429, 500].iter() {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let catalog = s.catalog();
        let past = s.past(vec![]);
        let page = s.timeline(
            INITIAL_DATE,
            100,
            vec![message(1, "picture", &s.server.url())],
        );
        let error = s
            .server
            .mock("GET", "/media/1")
            .with_status(*status)
            .with_body("error, not an image")
            .expect(1)
            .create();
        s.download().assert().code(1);
        assert!(files(&s.output()).is_empty());
        auth.assert();
        page.assert();
        past.assert();
        error.assert();
        for m in catalog {
            m.assert();
        }
    }
}

#[test]
fn interrupted_download_can_be_retried_without_accepting_partial_content() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(
        INITIAL_DATE,
        100,
        vec![message(1, "video", &s.server.url())],
    );
    let broken = s
        .server
        .mock("GET", "/media/1")
        .with_chunked_body(|writer| {
            writer.write_all(b"partial")?;
            Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "simulated disconnect",
            ))
        })
        .expect(1)
        .create();
    s.download().assert().code(1);
    assert!(!s
        .member_dir()
        .join("1_2_20260923010203_unknown.mp4")
        .exists());
    broken.assert();
    broken.remove();
    let media = s.media("/media/1");
    s.download().assert().success();
    assert_eq!(
        fs::read(s.member_dir().join("1_2_20260923010203_unknown.mp4")).unwrap(),
        MEDIA
    );
    auth.assert();
    media.assert();
    drop((catalog, past, page));
}

#[test]
fn fully_filtered_page_still_advances_to_later_messages() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let mut messages: Vec<_> = (1..=100)
        .map(|id| message(id, "text", &s.server.url()))
        .collect();
    messages[0]["updated_at"] = json!("2026-09-22T01:02:03Z");
    let first = s.timeline(INITIAL_DATE, 100, messages);
    let second = s.timeline(DATE, 100, vec![message(101, "picture", &s.server.url())]);
    let media = s.media("/media/101");
    s.download().args(["--kind", "picture"]).assert().success();
    assert_eq!(
        fs::read(s.member_dir().join("101_1_20260923010203_unknown.jpg")).unwrap(),
        MEDIA
    );
    assert_eq!(files(&s.output()).len(), 2);
    auth.assert();
    first.assert();
    second.assert();
    media.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn persistent_unauthorized_response_stops_for_every_service() {
    for service in SERVICES {
        let mut s = Scenario::new(service);
        let auth = s.auth();
        let failure = s
            .server
            .mock("GET", "/v2/groups?")
            .with_status(401)
            .expect_at_least(1)
            .expect_at_most(2)
            .create();
        // A killed/timed-out child is NOT an acceptable error exit.
        let output = s.download().output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(1),
            "must exit normally after bounded retries"
        );
        failure.assert();
        drop(auth);
    }
}

#[test]
fn malformed_configuration_is_reported_instead_of_silently_ignored() {
    let s = Scenario::new(SERVICES[0]);
    fs::write(s.config_file(), "--s_refresh_token 'unterminated").unwrap();
    s.command().assert().code(1);
}

#[test]
fn all_six_services_use_their_own_tokens_in_one_run_and_on_restart() {
    let mut services: Vec<_> = SERVICES
        .iter()
        .map(|service| Scenario::new(*service))
        .collect();
    let mut auth = Vec::new();
    let mut requests = Vec::new();
    for s in &mut services {
        auth.push(s.auth());
        requests.push(s.get("/v2/groups", json!([group(1, s.service.group, true, &[])])));
        requests.push(s.get("/v2/tags", json!([])));
        requests.push(s.past(vec![]));
        requests.push(s.timeline(INITIAL_DATE, 100, vec![message(1, "text", &s.server.url())]));
    }
    let download = |services: &[Scenario]| {
        let mut command = services[0].download();
        for s in services.iter().skip(1) {
            command.args([s.service.token_flag, "test-refresh-token"]);
            command.env(s.service.base_env, s.server.url());
        }
        command
    };
    download(&services).assert().success();
    for m in &requests {
        m.assert();
    }
    for s in &mut services {
        s.timeline(DATE, 100, vec![message(1, "text", &s.server.url())]);
    }
    download(&services).assert().success();
    for m in auth {
        m.assert();
    }
    assert_eq!(files(&services[0].output()).len(), 6);
    for s in &services {
        assert_eq!(
            fs::read_to_string(
                services[0]
                    .output()
                    .join(s.service.group)
                    .join("1_0_20260923010203_unknown.txt")
            )
            .unwrap(),
            format!("{}\n", TEXT)
        );
    }
}

#[test]
fn repeated_message_between_past_and_timeline_is_not_overwritten_or_redownloaded() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![message(1, "picture", &s.server.url())]);
    let mut duplicate = message(1, "picture", &s.server.url());
    duplicate["text"] = json!("must not overwrite existing content");
    let page = s.timeline(INITIAL_DATE, 100, vec![duplicate]);
    let media = s.media("/media/1");
    s.download().assert().success();
    assert_eq!(
        fs::read_to_string(s.member_dir().join("1_1_20260923010203_unknown.txt")).unwrap(),
        format!("{}\n", TEXT)
    );
    auth.assert();
    past.assert();
    page.assert();
    media.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn nonadvancing_timeline_is_reported_instead_of_retried_forever() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let mut messages: Vec<_> = (1..=100)
        .map(|id| message(id, "text", &s.server.url()))
        .collect();
    messages[0]["updated_at"] = json!("2026-09-22T01:02:03Z");
    let page = s.timeline(DATE, 100, messages);
    let output = s
        .download()
        .args(["--from", "2026/09/23 01:02:03"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("timeline did not advance"));
    auth.assert();
    past.assert();
    page.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn media_destination_failure_cleans_up_temporary_download() {
    let mut s = Scenario::new(SERVICES[0]);
    fs::create_dir_all(s.member_dir().join("1_2_20260923010203_unknown.mp4")).unwrap();
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(
        INITIAL_DATE,
        100,
        vec![message(1, "video", &s.server.url())],
    );
    let media = s.media("/media/1");
    s.download().assert().code(1);
    assert!(files(&s.output()).is_empty());
    auth.assert();
    past.assert();
    page.assert();
    media.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn unauthorized_token_refresh_is_bounded_without_an_existing_cache() {
    for service in SERVICES {
        let mut s = Scenario::new(service);
        let error = s
            .server
            .mock("POST", "/v2/update_token")
            .with_status(401)
            .expect(2)
            .create();
        s.download().assert().code(1);
        assert!(!s.token_file().exists());
        error.assert();
    }
}
