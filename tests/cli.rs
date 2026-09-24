mod common;

use common::*;
use serde_json::json;

const STAMP: &str = "20260923010203";
use std::{fs, path::PathBuf};

fn saves_service(service: Service) {
    let mut s = Scenario::new(service);
    let auth = s.auth();
    let catalog = s.catalog();
    let base = s.server.url();
    let past = s.past(vec![message(1, "text", &base)]);
    let page = s.timeline(
        INITIAL_DATE,
        100,
        vec![
            message(2, "picture", &base),
            message(3, "video", &base),
            message(4, "voice", &base),
            message(5, "link", &base),
        ],
    );
    let media: Vec<_> = (2..=4)
        .map(|id| s.media(&format!("/media/{}", id)))
        .collect();
    s.download().assert().success();
    assert_eq!(
        files(&s.output()),
        vec![
            "一期生/テストメンバー/1_0_20260923010203.txt",
            "一期生/テストメンバー/2_1_20260923010203.jpg",
            "一期生/テストメンバー/2_1_20260923010203.txt",
            "一期生/テストメンバー/3_2_20260923010203.mp4",
            "一期生/テストメンバー/4_3_20260923010203.mp4",
            "一期生/テストメンバー/5_4_20260923010203.txt",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>()
    );
    for name in &["1_0", "2_1", "5_4"] {
        assert_eq!(
            fs::read_to_string(s.member_dir().join(format!("{}_{}.txt", name, STAMP))).unwrap(),
            format!("{}\n", TEXT)
        );
    }
    for name in &[
        "2_1_20260923010203.jpg",
        "3_2_20260923010203.mp4",
        "4_3_20260923010203.mp4",
    ] {
        assert_eq!(fs::read(s.member_dir().join(name)).unwrap(), MEDIA);
    }
    auth.assert();
    for mock in catalog.iter().chain(media.iter()) {
        mock.assert();
    }
    past.assert();
    page.assert();

    // A second run must reuse the token and preserve existing content without downloading media.
    let snapshots: Vec<_> = files(&s.output())
        .iter()
        .map(|p| (p.clone(), fs::read(s.output().join(p)).unwrap()))
        .collect();
    let next = s.timeline(
        DATE,
        100,
        (1..=5)
            .zip(["text", "picture", "video", "voice", "link"].iter())
            .map(|(id, kind)| message(id, kind, &base))
            .collect(),
    );
    s.download().assert().success();
    next.assert();
    auth.assert();
    for mock in &media {
        mock.assert();
    }
    assert_eq!(files(&s.output()).len(), snapshots.len());
    for (path, bytes) in snapshots {
        assert_eq!(fs::read(s.output().join(path)).unwrap(), bytes);
    }
}

#[test]
fn sakurazaka_messages_are_downloaded_and_saved() {
    saves_service(SERVICES[0]);
}
#[test]
fn hinatazaka_messages_are_downloaded_and_saved() {
    saves_service(SERVICES[1]);
}
#[test]
fn nogizaka_messages_are_downloaded_and_saved() {
    saves_service(SERVICES[2]);
}
#[test]
fn asukasaito_messages_are_downloaded_and_saved() {
    saves_service(SERVICES[3]);
}
#[test]
fn maishiraishi_messages_are_downloaded_and_saved() {
    saves_service(SERVICES[4]);
}
#[test]
fn yodel_messages_are_downloaded_and_saved() {
    saves_service(SERVICES[5]);
}

#[test]
fn only_selected_service_is_contacted() {
    for service in SERVICES {
        let mut s = Scenario::new(service);
        let auth = s.auth();
        let groups = s.get("/v2/groups", json!([]));
        let tags = s.get("/v2/tags", json!([]));
        let mut command = s.download();
        for other in SERVICES.iter().filter(|other| other.group != service.group) {
            command.args([other.token_flag, "other-refresh-token"]);
            command.env(other.base_env, "http://127.0.0.1:1");
        }
        command.args(["--group", service.group]).assert().success();
        auth.assert();
        groups.assert();
        tags.assert();
        assert!(files(&s.output()).is_empty());
    }
}

#[test]
fn help_version_and_directory_options_do_not_contact_the_api() {
    let s = Scenario::new(SERVICES[0]);
    for (arg, expected) in &[
        ("--help", "--s_refresh_token"),
        ("--version", env!("CARGO_PKG_VERSION")),
    ] {
        let output = s.command().arg(arg).output().unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
    let output = s.command().arg("--config-dir").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(output.stdout).unwrap().trim()),
        s.root.path().join("config/colmsg")
    );
    let output = s.command().arg("--download-dir").output().unwrap();
    assert!(output.status.success());
    let download_dir = PathBuf::from(String::from_utf8(output.stdout).unwrap().trim());
    assert!(download_dir.is_absolute());
    assert!(download_dir.ends_with("colmsg"));
    s.command().assert().success();
}

#[test]
fn invalid_cli_arguments_fail_without_network() {
    for args in &[
        vec!["--unknown"],
        vec!["--kind", "invalid"],
        vec!["--group", "invalid"],
        vec!["--s_refresh_token"],
    ] {
        Scenario::new(SERVICES[0])
            .command()
            .args(args)
            .assert()
            .code(1);
    }
    Scenario::new(SERVICES[0])
        .download()
        .args(["--from", "not-a-date"])
        .assert()
        .code(1);
}

#[test]
fn configuration_supports_comments_quoted_names_and_dates() {
    let mut s = Scenario::new(SERVICES[0]);
    fs::write(s.config_file(), "\n# comment\n --s_refresh_token test-refresh-token\n--name 'テストメンバー'\n--from '2026/09/20 12:34:56'\n--kind text\n").unwrap();
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(
        "2026-09-20T12:34:56Z",
        100,
        vec![message(1, "text", &s.server.url())],
    );
    s.command().arg("--dir").arg(s.output()).assert().success();
    assert_eq!(files(&s.output()).len(), 1);
    auth.assert();
    past.assert();
    page.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn missing_config_override_falls_back_to_default_config() {
    let mut s = Scenario::new(SERVICES[0]);
    fs::write(s.config_file(), "--s_refresh_token test-refresh-token").unwrap();
    let auth = s.auth();
    let groups = s.get("/v2/groups", json!([]));
    let tags = s.get("/v2/tags", json!([]));
    s.command()
        .env("COLMSG_CONFIG_PATH", s.root.path().join("missing"))
        .arg("--dir")
        .arg(s.output())
        .assert()
        .success();
    auth.assert();
    groups.assert();
    tags.assert();
}

#[test]
fn missing_configuration_is_allowed() {
    let s = Scenario::new(SERVICES[0]);
    fs::remove_file(s.config_file()).unwrap();
    s.command().assert().success();
}

#[test]
fn names_subscriptions_and_optional_tag_metadata_are_respected() {
    let mut s = Scenario::new(SERVICES[2]);
    let auth = s.auth();
    let groups = s.get(
        "/v2/groups",
        json!([
            group(1, "テスト　メンバー", true, &[]),
            group(2, "非購読", false, &[]),
            group(3, "対象外", true, &["group", "generation"])
        ]),
    );
    let tags = s.get(
        "/v2/tags",
        json!([
            tag("group", "グループ", json!({"dimension":"group"})),
            tag("generation", "一期生", json!(null))
        ]),
    );
    let past = s.past(vec![]);
    let page = s.timeline(INITIAL_DATE, 100, vec![message(1, "text", &s.server.url())]);
    s.download()
        .args(["--name", " テストメンバー "])
        .assert()
        .success();
    assert_eq!(
        files(&s.output()),
        vec![PathBuf::from("テストメンバー/1_0_20260923010203.txt")]
    );
    auth.assert();
    groups.assert();
    tags.assert();
    past.assert();
    page.assert();
}

#[test]
fn every_kind_filter_saves_only_that_kind() {
    for (index, kind) in ["text", "picture", "video", "voice", "link"]
        .iter()
        .enumerate()
    {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let catalog = s.catalog();
        let past = s.past(vec![]);
        let base = s.server.url();
        let messages = ["text", "picture", "video", "voice", "link"]
            .iter()
            .enumerate()
            .map(|(i, k)| message(i as u32 + 1, k, &base))
            .collect();
        let page = s.timeline(INITIAL_DATE, 100, messages);
        let media = if (1..=3).contains(&index) {
            Some(s.media(&format!("/media/{}", index + 1)))
        } else {
            None
        };
        s.download().args(["--kind", kind]).assert().success();
        let actual = files(&s.member_dir());
        assert_eq!(actual.len(), if *kind == "picture" { 2 } else { 1 });
        assert!(actual
            .iter()
            .all(|p| p
                .to_str()
                .unwrap()
                .starts_with(&format!("{}_{}_", index + 1, index))));
        auth.assert();
        page.assert();
        past.assert();
        for m in catalog {
            m.assert();
        }
        if let Some(m) = media {
            m.assert();
        }
    }
}

#[test]
fn absent_optional_text_and_media_do_not_create_empty_files() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let messages = ["text", "picture", "video", "voice", "link"]
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let mut m = message(i as u32, k, &s.server.url());
            m["text"] = json!(null);
            m["file"] = json!(null);
            m
        })
        .collect();
    let page = s.timeline(INITIAL_DATE, 100, messages);
    s.download().assert().success();
    assert!(files(&s.output()).is_empty());
    auth.assert();
    page.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn literal_escaped_line_breaks_are_normalized() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let mut m = message(1, "text", &s.server.url());
    m["text"] = json!("a\\r\\nb");
    let page = s.timeline(INITIAL_DATE, 100, vec![m]);
    s.download().assert().success();
    assert_eq!(
        fs::read_to_string(s.member_dir().join("1_0_20260923010203.txt")).unwrap(),
        "a\nb\n"
    );
    auth.assert();
    page.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn unknown_kind_and_invalid_timestamp_fail_without_saving() {
    for (kind, date, error) in &[
        ("unknown", DATE, "unknown type"),
        ("text", "invalid", "Parse error"),
    ] {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let catalog = s.catalog();
        let past = s.past(vec![]);
        let mut m = message(1, kind, &s.server.url());
        m["updated_at"] = json!(date);
        let page = s.timeline(INITIAL_DATE, 100, vec![m]);
        let output = s.download().output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains(error));
        assert!(files(&s.output()).is_empty());
        auth.assert();
        page.assert();
        past.assert();
        for m in catalog {
            m.assert();
        }
    }
}

#[test]
fn empty_and_short_pages_finish() {
    for count in [0, 1, 99].iter() {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let catalog = s.catalog();
        let past = s.past(vec![]);
        let messages = (1..=*count)
            .map(|id| message(id, "text", &s.server.url()))
            .collect();
        let page = s.timeline(INITIAL_DATE, 100, messages);
        s.download().assert().success();
        assert_eq!(files(&s.output()).len(), *count as usize);
        auth.assert();
        page.assert();
        past.assert();
        for m in catalog {
            m.assert();
        }
    }
}

#[test]
fn full_page_advances_date_and_preserves_overlapping_messages() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let mut messages: Vec<_> = (1..=100)
        .map(|id| message(id, "text", &s.server.url()))
        .collect();
    messages[0]["updated_at"] = json!("2026-09-22T01:02:03Z");
    let first = s.timeline(INITIAL_DATE, 100, messages);
    let second = s.timeline(
        DATE,
        100,
        vec![
            message(100, "text", &s.server.url()),
            message(101, "text", &s.server.url()),
        ],
    );
    s.download().assert().success();
    assert_eq!(files(&s.output()).len(), 101);
    auth.assert();
    first.assert();
    second.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn equal_timestamps_expand_page_size_without_losing_messages() {
    for length in [100, 101, 200, 201].iter() {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let catalog = s.catalog();
        let past = s.past(vec![]);
        let mut pages = Vec::new();
        for count in (100..=((*length / 100 + 1) * 100)).step_by(100) {
            pages.push(
                s.timeline(
                    INITIAL_DATE,
                    count,
                    (1..=count.min(*length))
                        .map(|id| message(id as u32, "text", &s.server.url()))
                        .collect(),
                ),
            );
        }
        s.download().assert().success();
        assert_eq!(files(&s.output()).len(), *length);
        auth.assert();
        past.assert();
        for m in catalog.into_iter().chain(pages) {
            m.assert();
        }
    }
}

#[test]
fn resumes_from_saved_files_and_ignores_unrelated_files() {
    let mut s = Scenario::new(SERVICES[0]);
    fs::create_dir_all(s.member_dir()).unwrap();
    fs::write(s.member_dir().join("8_0_20260923010203.txt"), "keep me").unwrap();
    fs::write(s.member_dir().join("notes.txt"), "unrelated").unwrap();
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![message(8, "text", &s.server.url())]);
    let page = s.timeline(
        DATE,
        100,
        vec![
            message(8, "text", &s.server.url()),
            message(9, "text", &s.server.url()),
        ],
    );
    s.download().assert().success();
    assert_eq!(
        fs::read_to_string(s.member_dir().join("8_0_20260923010203.txt")).unwrap(),
        "keep me"
    );
    assert_eq!(
        fs::read_to_string(s.member_dir().join("notes.txt")).unwrap(),
        "unrelated"
    );
    assert_eq!(files(&s.output()).len(), 3);
    auth.assert();
    past.assert();
    page.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn http_failures_are_reported_for_each_service() {
    for service in SERVICES {
        for status in [403, 429, 500].iter() {
            let mut s = Scenario::new(service);
            let auth = s.auth();
            let failure = s
                .server
                .mock("GET", "/v2/groups?")
                .with_status(*status)
                .expect(1)
                .create();
            let output = s.download().output().unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&output.stderr).contains(&status.to_string()));
            auth.assert();
            failure.assert();
            assert!(files(&s.output()).is_empty());
        }
    }
}

#[test]
fn malformed_or_incompatible_api_json_is_reported() {
    for body in &["not-json", "{}", "[{}]"] {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let failure = s
            .server
            .mock("GET", "/v2/groups?")
            .with_body(*body)
            .expect(1)
            .create();
        let output = s.download().output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("/v2/groups"));
        auth.assert();
        failure.assert();
    }
}

#[test]
fn token_refresh_failure_is_reported_for_each_service() {
    for service in SERVICES {
        let mut s = Scenario::new(service);
        let failure = s
            .server
            .mock("POST", "/v2/update_token")
            .with_status(400)
            .expect(1)
            .create();
        s.download().assert().code(1);
        failure.assert();
    }
}

#[test]
fn expired_cached_token_is_refreshed_for_each_service() {
    for service in SERVICES {
        let mut s = Scenario::new(service);
        fs::write(s.token_file(), "expired").unwrap();
        let expired = s
            .server
            .mock("GET", "/v2/groups?")
            .match_header("authorization", "Bearer expired")
            .with_status(401)
            .expect(1)
            .create();
        let auth = s.auth();
        let groups = s.get("/v2/groups", json!([]));
        let tags = s.get("/v2/tags", json!([]));
        s.download().assert().success();
        expired.assert();
        auth.assert();
        groups.assert();
        tags.assert();
    }
}

#[test]
fn transport_and_invalid_url_errors_are_reported() {
    for url in &["http://127.0.0.1:1", "not a URL"] {
        let s = Scenario::new(SERVICES[0]);
        s.download().env(s.service.base_env, url).assert().code(1);
    }
}

#[test]
fn invalid_cached_authorization_is_rejected() {
    let s = Scenario::new(SERVICES[0]);
    fs::write(s.token_file(), "bad\ntoken").unwrap();
    s.download().assert().code(1);
}

#[test]
fn filesystem_errors_are_reported() {
    let s = Scenario::new(SERVICES[0]);
    fs::write(s.output(), "not a directory").unwrap();
    s.download().assert().code(1);

    let mut s = Scenario::new(SERVICES[0]);
    fs::create_dir_all(s.output()).unwrap();
    fs::write(s.output().join("一期生"), "not a directory").unwrap();
    let auth = s.auth();
    let catalog = s.catalog();
    s.download().assert().code(1);
    auth.assert();
    for m in catalog {
        m.assert();
    }

    let mut s = Scenario::new(SERVICES[0]);
    fs::create_dir_all(s.member_dir().join("1_0_20260923010203.txt")).unwrap();
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(INITIAL_DATE, 100, vec![message(1, "text", &s.server.url())]);
    s.download().assert().code(1);
    auth.assert();
    past.assert();
    page.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn invalid_date_in_existing_filename_is_reported() {
    let mut s = Scenario::new(SERVICES[0]);
    fs::create_dir_all(s.member_dir()).unwrap();
    fs::write(s.member_dir().join("1_0_20261399010203.txt"), "keep").unwrap();
    let auth = s.auth();
    let catalog = s.catalog();
    s.download().assert().code(1);
    auth.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn failures_at_each_api_stage_stop_without_saving() {
    for path in &[
        "/v2/tags?",
        "/v2/groups/1/past_messages?order=asc",
        "/v2/groups/1/timeline",
    ] {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let mut mocks = vec![s.get(
            "/v2/groups",
            json!([group(1, "テスト メンバー", true, &["generation"])]),
        )];
        if *path != "/v2/tags?" {
            mocks.push(s.get(
                "/v2/tags",
                json!([tag("generation", "一期生", json!(null))]),
            ));
        }
        if *path == "/v2/groups/1/timeline" {
            mocks.push(s.past(vec![]));
        }
        let failure = s
            .server
            .mock("GET", path.split('?').next().unwrap())
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .expect(1)
            .create();
        s.download().assert().code(1);
        assert!(files(&s.output()).is_empty());
        auth.assert();
        failure.assert();
        for m in mocks {
            m.assert();
        }
    }
}

#[test]
fn response_can_add_fields_and_omit_optional_fields() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let mut m = message(1, "text", &s.server.url());
    m["future_server_field"] = json!({"nested":[true, 1]});
    let page = s.timeline(INITIAL_DATE, 100, vec![m]);
    s.download().assert().success();
    assert_eq!(
        fs::read_to_string(s.member_dir().join("1_0_20260923010203.txt")).unwrap(),
        format!("{}\n", TEXT)
    );
    auth.assert();
    page.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn media_download_follows_redirects_without_api_authorization() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(
        INITIAL_DATE,
        100,
        vec![message(1, "picture", &s.server.url())],
    );
    let mut cdn = mockito::Server::new();
    let redirect = s
        .server
        .mock("GET", "/media/1")
        .with_status(302)
        .with_header("location", &format!("{}/image", cdn.url()))
        .expect(1)
        .create();
    let image = cdn
        .mock("GET", "/image")
        .match_header("authorization", mockito::Matcher::Missing)
        .with_body(MEDIA)
        .expect(1)
        .create();
    s.download().assert().success();
    assert_eq!(
        fs::read(s.member_dir().join("1_1_20260923010203.jpg")).unwrap(),
        MEDIA
    );
    auth.assert();
    page.assert();
    past.assert();
    redirect.assert();
    image.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn invalid_media_url_fails_without_creating_a_file() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let mut m = message(1, "video", &s.server.url());
    m["file"] = json!("not a URL");
    let page = s.timeline(INITIAL_DATE, 100, vec![m]);
    s.download().assert().code(1);
    assert!(files(&s.output()).is_empty());
    auth.assert();
    page.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn empty_text_is_preserved_as_a_message() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let mut m = message(1, "text", &s.server.url());
    m["text"] = json!("");
    let page = s.timeline(INITIAL_DATE, 100, vec![m]);
    s.download().assert().success();
    assert_eq!(
        fs::read(s.member_dir().join("1_0_20260923010203.txt")).unwrap(),
        b"\n"
    );
    auth.assert();
    page.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[test]
fn custom_configuration_file_is_used() {
    let mut s = Scenario::new(SERVICES[0]);
    let custom = s.root.path().join("custom.conf");
    fs::write(&custom, "--s_refresh_token test-refresh-token").unwrap();
    let auth = s.auth();
    let groups = s.get("/v2/groups", json!([]));
    let tags = s.get("/v2/tags", json!([]));
    s.command()
        .env("COLMSG_CONFIG_PATH", custom)
        .arg("--dir")
        .arg(s.output())
        .assert()
        .success();
    auth.assert();
    groups.assert();
    tags.assert();
}

#[test]
fn uncreatable_token_directory_is_reported() {
    let s = Scenario::new(SERVICES[0]);
    fs::remove_dir_all(s.root.path().join("config/colmsg")).unwrap();
    fs::write(s.root.path().join("config/colmsg"), "not a directory").unwrap();
    s.download().assert().code(1);
}

#[test]
fn token_cache_file_creation_failure_is_reported() {
    let mut s = Scenario::new(SERVICES[0]);
    fs::create_dir(s.token_file()).unwrap();
    let auth = s.auth();

    s.download().assert().code(1);

    auth.assert();
    assert!(s.token_file().is_dir());
}

#[test]
fn token_cache_is_created_when_configuration_directory_is_missing() {
    let mut s = Scenario::new(SERVICES[0]);
    fs::remove_dir_all(s.root.path().join("config/colmsg")).unwrap();
    let auth = s.auth();
    let groups = s.get("/v2/groups", json!([]));
    let tags = s.get("/v2/tags", json!([]));
    s.download().assert().success();
    // Reusing the cache must avoid a second token request.
    s.download().assert().success();
    auth.assert();
    assert!(s.root.path().join("config/colmsg").is_dir());
    drop((groups, tags));
}

#[cfg(target_os = "macos")]
#[test]
fn config_directory_falls_back_to_isolated_home_when_xdg_is_unset_or_relative() {
    let s = Scenario::new(SERVICES[0]);
    for relative in [false, true].iter() {
        let mut command = s.command();
        command.env_remove("COLMSG_CONFIG_DIR");
        command.env_remove("XDG_CONFIG_HOME");
        if *relative {
            command.env("XDG_CONFIG_HOME", "relative/config");
        }
        let output = command.arg("--config-dir").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            PathBuf::from(String::from_utf8(output.stdout).unwrap().trim()),
            s.root.path().join("home/.config/colmsg")
        );
    }
}

#[test]
fn service_works_with_only_its_own_api_override() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(INITIAL_DATE, 100, vec![message(1, "text", &s.server.url())]);
    let mut command = s.download();
    for other in SERVICES.iter().skip(1) {
        command.env_remove(other.base_env);
    }
    command.assert().success();
    assert_eq!(
        fs::read_to_string(s.member_dir().join("1_0_20260923010203.txt")).unwrap(),
        format!("{}\n", TEXT)
    );
    auth.assert();
    page.assert();
    past.assert();
    for m in catalog {
        m.assert();
    }
}

#[cfg(unix)]
#[test]
fn default_download_directory_is_created_under_isolated_home() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let groups = s.get("/v2/groups", json!([]));
    let tags = s.get("/v2/tags", json!([]));
    s.command()
        .args([s.service.token_flag, "test-refresh-token"])
        .assert()
        .success();
    assert!(s.root.path().join("home/Downloads/colmsg").is_dir());
    auth.assert();
    groups.assert();
    tags.assert();
}

#[test]
fn gzip_encoded_api_response_is_decoded() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let groups = s
        .server
        .mock("GET", "/v2/groups?")
        .with_header("content-encoding", "gzip")
        .with_header("content-type", "application/json")
        .with_body(include_bytes!("fixtures/empty-array.json.gz").as_ref())
        .expect(1)
        .create();
    let tags = s.get("/v2/tags", json!([]));
    s.download().assert().success();
    assert!(files(&s.output()).is_empty());
    auth.assert();
    groups.assert();
    tags.assert();
}

#[test]
fn stalled_api_response_times_out_without_retrying_or_saving() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let stalled = s
        .server
        .mock("GET", "/v2/groups?")
        .with_chunked_body(|writer| {
            writer.write_all(b"[")?;
            // Exercise reqwest's real 30-second deadline, rather than a test-only timeout.
            std::thread::sleep(std::time::Duration::from_secs(31));
            writer.write_all(b"]")
        })
        .expect(1)
        .create();
    let output = s
        .download()
        .timeout(std::time::Duration::from_secs(40))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("timed out"));
    assert!(files(&s.output()).is_empty());
    auth.assert();
    stalled.assert();
}
