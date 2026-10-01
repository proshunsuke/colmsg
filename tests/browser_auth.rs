#[allow(dead_code)]
mod common;
use common::*;
use mockito::{Matcher, Mock};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::OnceLock};

const HOSTS: [&str; 6] = [
    "message.sakurazaka46.com",
    "message.hinatazaka46.com",
    "message.nogizaka46.com",
    "message.asukasaito.jp",
    "message.maishiraishi-official.com",
    "service.yodel-app.com",
];
const WEB_ID: &str = "web-client 1.0";

fn fixture() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        escargot::CargoBuild::new()
            .manifest_path(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .test("browser_fixture")
            .run_tests()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .to_owned()
    })
}
fn host(s: &Scenario) -> &'static str {
    HOSTS[SERVICES
        .iter()
        .position(|v| v.group == s.service.group)
        .unwrap()]
}
fn auth_path(s: &Scenario) -> PathBuf {
    s.root
        .path()
        .join(format!("config/colmsg/auth/{}.json", s.service.group))
}
fn credential(s: &Scenario, expires: i64) -> Value {
    json!({"version":1,"api_url":format!("https://{}/v2/update_token",host(s)),"headers":{"x-talk-app-id":WEB_ID,"x-talk-app-platform":"web","Accept":"application/json"},"cookies":[format!("session=old-secret; Domain={}; Path=/v2/update_token; Secure; HttpOnly",host(s))],"username":"test-user","access_token":format!("test-access-token-{}",s.service.group),"expires_at":expires,"cookie_expires_at":null})
}
fn write_credential(s: &Scenario, c: &Value) {
    fs::create_dir_all(auth_path(s).parent().unwrap()).unwrap();
    fs::write(auth_path(s), c.to_string()).unwrap();
}
fn saved(s: &Scenario) -> Value {
    serde_json::from_slice(&fs::read(auth_path(s)).unwrap()).unwrap()
}
fn scenario(service: Service) -> Scenario {
    Scenario::new(Service {
        app_id: WEB_ID,
        ..service
    })
}
fn download(s: &Scenario) -> assert_cmd::Command {
    let mut c = s.command();
    c.args(["-g", s.service.group, "--dir"]).arg(s.output());
    c
}
fn update(s: &mut Scenario, status: usize, body: Value, cookie: &str) -> Mock {
    let rotated = format!("session=rotated-secret; Domain={}; Path=/v2/update_token; Secure; HttpOnly; Expires=Wed, 01 Jan 2031 00:00:00 GMT", host(s));
    let logout = format!(
        "session=logout-only; Domain={}; Path=/v2/signout; Secure",
        host(s)
    );
    s.server
        .mock("POST", "/v2/update_token")
        .match_header("x-talk-app-id", WEB_ID)
        .match_header("x-talk-app-platform", "web")
        .match_header("authorization", Matcher::Missing)
        .match_header("cookie", cookie)
        .match_body(Matcher::Json(json!({"refresh_token":null})))
        .with_status(status)
        .with_header("content-type", "application/json")
        .with_header("set-cookie", &rotated)
        .with_header("set-cookie", &logout)
        .with_body(body.to_string())
        .expect(1)
        .create()
}

fn good_update(s: &mut Scenario) -> Mock {
    let body =
        json!({"access_token":format!("test-access-token-{}",s.service.group),"expires_in":3600});
    update(s, 200, body, "session=old-secret")
}
fn account(s: &mut Scenario, status: usize, body: Value) -> Mock {
    s.server
        .mock("GET", "/v2/account")
        .match_header("x-talk-app-id", WEB_ID)
        .match_header("cookie", Matcher::Missing)
        .match_header(
            "authorization",
            format!("Bearer test-access-token-{}", s.service.group).as_str(),
        )
        .with_status(status)
        .with_header("content-type", "application/json")
        .with_body(body.to_string())
        .expect(1)
        .create()
}
fn events(url: &str) -> Value {
    json!([
        {"method":"Network.requestWillBeSent","params":{"requestId":"auth","request":{"method":"POST","url":url,"headers":{"X-Talk-App-Id":WEB_ID,"x-talk-app-platform":"web","Origin":"https://official.example","Referer":"https://official.example/","User-Agent":"fixture browser","Accept-Language":"ja","Authorization":"SECRET google token","Cookie":"SECRET unrelated cookie","Ignored":123}}}},
        {"method":"Network.responseReceived","params":{"requestId":"auth","response":{"status":200}}},
        {"method":"Network.loadingFinished","params":{"requestId":"auth"}}
    ])
}
fn browser_scenario(s: &Scenario) -> Value {
    json!({"ping":true,"events":events(&format!("https://{}/v2/signin",host(s))),"cookies":[{"name":"session","value":"old-secret","domain":host(s),"path":"/v2/update_token","expires":1924992000.0},{"name":"google","value":"DO-NOT-SAVE","domain":"google.com","path":"/"}]})
}
fn login(s: &Scenario, data: Value) -> assert_cmd::Command {
    let file = s.root.path().join("browser.json");
    fs::write(&file, data.to_string()).unwrap();
    let mut c = s.command();
    c.args(["login", s.service.group, "--browser"])
        .arg(fixture());
    c.env("BROWSER_SCENARIO", file)
        .env("BROWSER_PROFILE", s.root.path().join("profile.txt"))
        .env("BROWSER_LOG", s.root.path().join("browser.log"));
    c
}
fn assert_profile_retained(s: &Scenario) {
    let profile = fs::read_to_string(s.root.path().join("profile.txt")).unwrap();
    assert!(
        PathBuf::from(profile).is_dir(),
        "browser profile must be retained for subsequent logins"
    );
}
fn assert_download(s: &mut Scenario) {
    let catalog = s.catalog();
    let past = s.past(vec![message(1, "text", &s.server.url())]);
    let page = s.timeline(INITIAL_DATE, 100, vec![]);
    download(s).assert().success();
    assert_eq!(
        fs::read_to_string(s.member_dir().join("1_0_20260923010203_unknown.txt")).unwrap(),
        format!("{}\n", TEXT)
    );
    for m in catalog {
        m.assert()
    }
    past.assert();
    page.assert();
    s.assert_member_mocks();
}

fn browser_login_saves_messages(service: Service) {
    let mut s = scenario(service);
    let data = browser_scenario(&s);
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    let out = login(&s, data).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    refresh.assert();
    who.assert();
    assert_profile_retained(&s);
    assert!(!s.output().exists(), "login must not save messages");
    let c = saved(&s);
    assert_eq!(c["username"], "test-user");
    assert_eq!(c["cookies"].as_array().unwrap().len(), 1);
    assert!(c["cookies"][0].as_str().unwrap().contains("rotated-secret"));
    assert_eq!(c["cookie_expires_at"], 1924992000_i64);
    assert!(!c.to_string().contains("SECRET"));
    assert!(!c.to_string().contains("DO-NOT-SAVE"));
    assert!(!c.to_string().contains("browser-only-token"));
    let log: Vec<Value> = fs::read_to_string(s.root.path().join("browser.log"))
        .unwrap()
        .lines()
        .map(|v| serde_json::from_str(v).unwrap())
        .collect();
    assert!(log.iter().any(|v| v["method"] == "Page.navigate"
        && v["params"]["url"] == format!("https://{}/", host(&s))));
    assert!(log.iter().any(|v| v["method"] == "Network.getCookies"
        && v["params"]["urls"] == json!([format!("https://{}/v2/update_token", host(&s))])));
    assert!(log.iter().any(|v| v["method"] == "Network.setBlockedURLs"
        && v["params"]["urls"]
            == json!(["*/v2/groups/*/timeline*", "*/v2/groups/*/past_messages*"])));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(auth_path(&s)).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(auth_path(&s).parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
    // Legacy configuration must not override the newly selected browser authentication.
    fs::write(
        s.config_file(),
        format!("{} ignored-native-secret", s.service.token_flag),
    )
    .unwrap();
    assert_download(&mut s);
}

#[test]
fn browser_sakurazaka() {
    browser_login_saves_messages(SERVICES[0]);
}

#[test]
fn browser_hinatazaka() {
    browser_login_saves_messages(SERVICES[1]);
}

#[test]
fn browser_nogizaka() {
    browser_login_saves_messages(SERVICES[2]);
}

#[test]
fn browser_asukasaito() {
    browser_login_saves_messages(SERVICES[3]);
}

#[test]
fn browser_maishiraishi() {
    browser_login_saves_messages(SERVICES[4]);
}

#[test]
fn browser_yodel() {
    browser_login_saves_messages(SERVICES[5]);
}

#[test]
fn expired_tokens_are_updated_for_every_service() {
    for service in SERVICES {
        let mut s = scenario(service);
        write_credential(&s, &credential(&s, 0));
        let refresh = good_update(&mut s);
        let who = account(&mut s, 200, json!({"username":"test-user"}));
        assert_download(&mut s);
        refresh.assert();
        who.assert();
        assert!(saved(&s)["expires_at"].as_i64().unwrap() > chrono::Utc::now().timestamp() + 3500);
    }
}
#[test]
fn cached_tokens_and_auth_status_never_refresh_or_expose_secrets() {
    let mut s = scenario(SERVICES[0]);
    write_credential(&s, &credential(&s, chrono::Utc::now().timestamp() + 3600));
    fs::write(s.config_file(), "--h_refresh_token native-secret").unwrap();
    let out = s.command().args(["auth", "status"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("sakurazaka: browser, account=test-user"));
    assert!(text.contains("hinatazaka: refresh-token"));
    assert!(text.contains("yodel: not configured"));
    assert!(!text.contains("secret"));
    assert!(!text.contains("test-access-token"));
    assert_download(&mut s);
}
#[test]
fn refresh_token_is_selected_without_browser_credentials_for_every_service() {
    for service in SERVICES {
        let mut s = Scenario::new(service);
        let auth = s.auth();
        let catalog = s.catalog();
        let past = s.past(vec![]);
        let page = s.timeline(INITIAL_DATE, 100, vec![]);
        s.download().args(["-g", service.group]).assert().success();
        auth.assert();
        for m in catalog {
            m.assert()
        }
        past.assert();
        page.assert();
    }
}
#[test]
fn missing_credentials_and_invalid_cli_choices_fail() {
    let s = scenario(SERVICES[0]);
    for service in SERVICES {
        let out = s.command().args(["-g", service.group]).output().unwrap();
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("colmsg login"));
    }
    for args in [
        vec!["login", "unknown"],
        vec!["auth"],
        vec!["auth", "unknown"],
        vec!["login"],
    ] {
        s.command().args(args).assert().failure();
    }
}
#[test]
fn invalid_saved_credentials_are_rejected_without_network_or_secrets() {
    let s = scenario(SERVICES[0]);
    for url in [
        "http://message.sakurazaka46.com/v2/update_token",
        "https://attacker.invalid/v2/update_token",
        "https://message.hinatazaka46.com/v2/update_token",
        "https://message.sakurazaka46.com:8443/v2/update_token",
        "https://user:SECRET@message.sakurazaka46.com/v2/update_token",
        "https://message.sakurazaka46.com/v2/signin",
        "https://message.sakurazaka46.com/v2/update_token?SECRET",
        "https://message.sakurazaka46.com/v2/update_token#SECRET",
    ] {
        let mut c = credential(&s, 0);
        c["api_url"] = json!(url);
        write_credential(&s, &c);
        let out = download(&s).output().unwrap();
        assert!(!out.status.success());
        assert!(!String::from_utf8_lossy(&out.stderr).contains("SECRET"));
    }
    let mut c = credential(&s, 0);
    c["version"] = json!(2);
    write_credential(&s, &c);
    download(&s).assert().failure();
    fs::write(auth_path(&s), "SECRET not-json").unwrap();
    let out = s.command().args(["auth", "status"]).output().unwrap();
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("SECRET"));
}
#[test]
fn failed_updates_leave_saved_credentials_unchanged() {
    for (status, body) in [
        (400, json!({})),
        (401, json!({})),
        (500, json!({})),
        (302, json!({})),
        (200, json!({"access_token":"","expires_in":3600})),
        (200, json!({"access_token":"SECRET","expires_in":0})),
        (200, json!({"access_token":"SECRET","expires_in":-1})),
        (200, json!({"access_token":"SECRET","expires_in":i64::MAX})),
        (200, json!({"access_token":"SECRET","expires_in":"3600"})),
    ] {
        let mut s = scenario(SERVICES[0]);
        let c = credential(&s, 0);
        write_credential(&s, &c);
        let refresh = update(&mut s, status, body, "session=old-secret");
        let out = download(&s).output().unwrap();
        assert!(!out.status.success());
        assert_eq!(saved(&s), c);
        assert!(!String::from_utf8_lossy(&out.stderr).contains("SECRET"));
        refresh.assert();
    }
}
#[test]
fn account_failure_retains_rotated_cookie_but_invalidates_cached_token() {
    for (status, body) in [
        (401, json!({})),
        (500, json!({})),
        (200, json!({})),
        (200, json!({"username":""})),
        (200, json!({"username":"different-user"})),
    ] {
        let mut s = scenario(SERVICES[0]);
        write_credential(&s, &credential(&s, 0));
        let refresh = good_update(&mut s);
        let who = account(&mut s, status, body);
        download(&s).assert().failure();
        refresh.assert();
        who.assert();
        let c = saved(&s);
        assert_eq!(c["expires_at"], 0);
        assert_eq!(c["username"], "test-user");
        assert!(c["cookies"][0].as_str().unwrap().contains("rotated-secret"));
        assert!(files(&s.output()).is_empty());
        let body = json!({"access_token":format!("test-access-token-{}",s.service.group),"expires_in":3600});
        let refresh = update(&mut s, 200, body, "session=rotated-secret");
        let who = account(&mut s, 200, json!({"username":"test-user"}));
        assert_download(&mut s);
        refresh.assert();
        who.assert();
    }
}
#[test]
fn browser_errors_do_not_overwrite_existing_credentials_and_retain_profile() {
    for mode in [
        "startup_exit",
        "targets",
        "missing_connection",
        "command_error",
        "close",
        "invalid_protocol",
        "encoded",
        "body",
        "cookies",
        "cookie_value",
        "cookie_domain",
        "cookie_path",
    ] {
        let s = scenario(SERVICES[0]);
        let c = credential(&s, 123);
        write_credential(&s, &c);
        let mut data = browser_scenario(&s);
        match mode {
            "targets" => data["targets"] = json!([]),
            "missing_connection" => data["targets"] = json!([{"type":"page"}]),
            "command_error" => data[mode] = json!("Network.enable"),
            "encoded" => data["body_result"] = json!({"base64Encoded":true}),
            "body" => data["body_result"] = json!({"body":"SECRET invalid response"}),
            "cookies" => data[mode] = json!([]),
            "cookie_value" => data["cookies"][0]["value"] = Value::Null,
            "cookie_domain" => data["cookies"][0]["domain"] = Value::Null,
            "cookie_path" => data["cookies"][0]["path"] = Value::Null,
            _ => data[mode] = json!(true),
        }
        let out = login(&s, data).output().unwrap();
        assert!(!out.status.success(), "{}", mode);
        let expected = match mode {
            "startup_exit" => "Browser exited during startup",
            "targets" => "Browser has no page target",
            "missing_connection" => "Browser has no debugging connection",
            "command_error" => "Browser command failed: Network.enable",
            "close" => "Browser closed before login completed",
            "invalid_protocol" => "Invalid browser protocol response",
            "encoded" => "Unexpected encoded authentication response",
            "body" => "Invalid signin response",
            "cookies" => "no session cookie was issued",
            "cookie_value" => "Invalid session cookie",
            "cookie_domain" => "Invalid cookie domain",
            "cookie_path" => "Invalid cookie path",
            _ => unreachable!(),
        };
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(expected),
            "{}: {}",
            mode,
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!String::from_utf8_lossy(&out.stderr).contains("SECRET"));
        assert_eq!(saved(&s), c);
        assert_profile_retained(&s);
    }
}
#[test]
fn browser_start_failure_is_reported() {
    let s = scenario(SERVICES[0]);
    s.command()
        .args(["login", s.service.group, "--browser", "/missing-browser"])
        .assert()
        .failure();
}
#[test]
fn login_lock_prevents_concurrent_authentication() {
    let s = scenario(SERVICES[0]);
    fs::create_dir_all(auth_path(&s).parent().unwrap()).unwrap();
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(auth_path(&s).with_extension("lock"))
        .unwrap();
    lock.lock().unwrap();
    let out = login(&s, browser_scenario(&s)).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("already in use"));
    assert!(!s.root.path().join("profile.txt").exists());
}

#[test]
fn unauthorized_message_requests_force_refresh_once_for_every_service() {
    for service in SERVICES {
        let mut s = scenario(service);
        let mut c = credential(&s, chrono::Utc::now().timestamp() + 3600);
        c["access_token"] = json!("rejected-token");
        write_credential(&s, &c);
        let rejected = s
            .server
            .mock("GET", "/v2/members?")
            .match_header("authorization", "Bearer rejected-token")
            .with_status(401)
            .expect(1)
            .create();
        let refresh = good_update(&mut s);
        let who = account(&mut s, 200, json!({"username":"test-user"}));
        assert_download(&mut s);
        rejected.assert();
        refresh.assert();
        who.assert();
    }
}
#[test]
fn second_unauthorized_response_does_not_loop_or_use_native_credentials() {
    let mut s = scenario(SERVICES[0]);
    write_credential(&s, &credential(&s, chrono::Utc::now().timestamp() + 3600));
    fs::write(
        s.config_file(),
        format!("{} native-secret", s.service.token_flag),
    )
    .unwrap();
    let rejected = s
        .server
        .mock("GET", "/v2/groups?")
        .with_status(401)
        .expect(2)
        .create();
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    download(&s).assert().failure();
    rejected.assert();
    refresh.assert();
    who.assert();
    assert!(!s.token_file().exists());
}
#[test]
fn session_deletion_is_rejected_even_if_another_cookie_remains() {
    let mut s = scenario(SERVICES[0]);
    let mut c = credential(&s, 0);
    c["cookies"].as_array_mut().unwrap().push(json!(format!(
        "other=unrelated; Domain={}; Path=/v2/update_token; Secure",
        host(&s)
    )));
    write_credential(&s, &c);
    let refresh = s
        .server
        .mock("POST", "/v2/update_token")
        .with_header("content-type", "application/json")
        .with_header(
            "set-cookie",
            &format!(
                "session=; Max-Age=0; Domain={}; Path=/v2/update_token; Secure",
                host(&s)
            ),
        )
        .with_body(json!({"access_token":"new-token","expires_in":3600}).to_string())
        .expect(1)
        .create();
    let out = download(&s).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("removed the session cookie"));
    assert_eq!(saved(&s), c);
    refresh.assert();
}
#[test]
fn near_expiry_refreshes_but_long_lived_cache_does_not() {
    let mut s = scenario(SERVICES[0]);
    write_credential(&s, &credential(&s, chrono::Utc::now().timestamp() + 59));
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    assert_download(&mut s);
    refresh.assert();
    who.assert();
}
#[test]
fn protocol_ignores_unrelated_and_failed_requests_and_tracks_concurrent_signins() {
    let mut s = scenario(SERVICES[0]);
    let mut data = browser_scenario(&s);
    let base = format!("https://{}/v2/signin", host(&s));
    let mut all = Vec::new();
    for (index, url, method) in [
        (0, "bad-url".to_owned(), "POST"),
        (1, "https://attacker.invalid/v2/signin".to_owned(), "POST"),
        (2, base.clone(), "GET"),
        (3, format!("https://{}/v2/account", host(&s)), "POST"),
        (4, base.clone(), "POST"),
    ] {
        let mut e = events(&url).as_array().unwrap().clone();
        for v in &mut e {
            v["params"]["requestId"] = json!(format!("ignored-{index}"));
        }
        e[0]["params"]["request"]["method"] = json!(method);
        if index == 4 {
            all.push(e.remove(0));
            all.push(json!({"method":"Network.loadingFailed","params":{"requestId":"ignored-4"}}));
        } else {
            all.extend(e);
        }
    }
    let mut a = events(&base).as_array().unwrap().clone();
    let mut b = a.clone();
    for v in &mut b {
        v["params"]["requestId"] = json!("other-auth");
    }
    all.push(a.remove(0));
    all.push(b.remove(0));
    all.push(a.remove(0));
    all.push(b.remove(0));
    all.push(a.remove(0));
    all.push(b.remove(0));
    data["events"] = json!(all);
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    login(&s, data).assert().success();
    refresh.assert();
    who.assert();
    assert_profile_retained(&s);
}
#[test]
fn signin_without_access_token_does_not_hide_later_valid_authentication() {
    let mut s = scenario(SERVICES[0]);
    let mut data = browser_scenario(&s);
    let mut all = data["events"].as_array().unwrap().clone();
    let mut next = all.clone();
    for v in &mut next {
        v["params"]["requestId"] = json!("second");
    }
    all.extend(next);
    data["events"] = json!(all);
    data["bodies"] = json!({"auth":{"body":"{}","base64Encoded":false}});
    data["cookies"][0]["expires"] = json!(-1);
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    login(&s, data).assert().success();
    refresh.assert();
    who.assert();
    assert_profile_retained(&s);
}
#[test]
fn browser_environment_selects_fixture_and_explicit_path_takes_precedence() {
    let mut s = scenario(SERVICES[0]);
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    let mut command = login(&s, browser_scenario(&s));
    command
        .env("COLMSG_BROWSER", "/missing-browser")
        .assert()
        .success();
    refresh.assert();
    who.assert();
    let mut s = scenario(SERVICES[0]);
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    let data = browser_scenario(&s);
    let file = s.root.path().join("browser.json");
    fs::write(&file, data.to_string()).unwrap();
    s.command()
        .args(["login", s.service.group])
        .env("COLMSG_BROWSER", fixture())
        .env("BROWSER_SCENARIO", file)
        .env("BROWSER_PROFILE", s.root.path().join("profile.txt"))
        .env("BROWSER_LOG", s.root.path().join("browser.log"))
        .assert()
        .success();
    refresh.assert();
    who.assert();
    assert_profile_retained(&s);
}

#[test]
fn all_six_browser_services_are_selected_and_saved_in_one_invocation() {
    let mut scenarios: Vec<_> = SERVICES.iter().copied().map(scenario).collect();
    let mut command = scenarios[0].command();
    let root = scenarios[0].root.path().to_owned();
    let output = scenarios[0].output();
    let mut mocks = Vec::new();
    for s in &mut scenarios {
        let c = credential(s, chrono::Utc::now().timestamp() + 3600);
        let path = root.join(format!("config/colmsg/auth/{}.json", s.service.group));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, c.to_string()).unwrap();
        command.env(s.service.base_env, s.server.url());
        mocks.push(s.get("/v2/groups", json!([group(1, s.service.group, true, &[])])));
        mocks.push(s.get("/v2/tags", json!([])));
        mocks.push(s.past(vec![message(1, "text", &s.server.url())]));
        mocks.push(s.timeline(INITIAL_DATE, 100, vec![]));
    }
    command.arg("--dir").arg(&output).assert().success();
    assert_eq!(files(&output).len(), 6);
    for s in &scenarios {
        assert_eq!(
            fs::read_to_string(
                output
                    .join(s.service.group)
                    .join("1_0_20260923010203_unknown.txt")
            )
            .unwrap(),
            format!("{}\n", TEXT)
        );
        s.assert_member_mocks();
    }
    for m in mocks {
        m.assert();
    }
}

#[test]
fn invalid_web_headers_fail_before_saving_messages() {
    for headers in [
        json!({"invalid header":"x"}),
        json!({"x-talk-app-id":"bad\nvalue"}),
    ] {
        let s = scenario(SERVICES[0]);
        let mut c = credential(&s, chrono::Utc::now().timestamp() + 3600);
        c["headers"] = headers;
        write_credential(&s, &c);
        download(&s).assert().failure();
        assert!(files(&s.output()).is_empty());
    }
}

#[test]
fn alternate_official_api_hosts_work_for_every_service() {
    for service in SERVICES {
        for alternate in [
            format!(
                "api.{}",
                HOSTS[SERVICES
                    .iter()
                    .position(|s| s.group == service.group)
                    .unwrap()]
            ),
            format!(
                "api.{}",
                HOSTS[SERVICES
                    .iter()
                    .position(|s| s.group == service.group)
                    .unwrap()]
                .split_once('.')
                .unwrap()
                .1
            ),
        ] {
            let mut s = scenario(service);
            let mut c = credential(&s, chrono::Utc::now().timestamp() + 3600);
            c["api_url"] = json!(format!("https://{}/v2/update_token", alternate));
            write_credential(&s, &c);
            assert_download(&mut s);
        }
    }
}

#[test]
fn authentication_and_account_invalid_json_are_sanitized() {
    for invalid_account in [false, true] {
        let mut s = scenario(SERVICES[0]);
        let c = credential(&s, 0);
        write_credential(&s, &c);
        let good = if invalid_account {
            Some(good_update(&mut s))
        } else {
            None
        };
        let invalid = s
            .server
            .mock(
                if invalid_account { "GET" } else { "POST" },
                if invalid_account {
                    "/v2/account"
                } else {
                    "/v2/update_token"
                },
            )
            .with_header("content-type", "application/json")
            .with_body("SECRET invalid-json")
            .expect(1)
            .create();
        let out = download(&s).output().unwrap();
        assert!(!out.status.success());
        assert!(!String::from_utf8_lossy(&out.stderr).contains("SECRET"));
        invalid.assert();
        if let Some(m) = good {
            m.assert();
            assert_eq!(saved(&s)["expires_at"], 0);
        } else {
            assert_eq!(saved(&s), c);
        }
    }
}

#[test]
fn status_handles_unknown_expiry_without_network() {
    let s = scenario(SERVICES[0]);
    let mut c = credential(&s, i64::MAX);
    c["cookie_expires_at"] = json!(i64::MAX);
    write_credential(&s, &c);
    let out = s.command().args(["auth", "status"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("access_expires=unknown, cookie_expires=unknown"));
}

#[test]
fn browser_is_discovered_from_path_without_launching_an_installed_browser() {
    let mut s = scenario(SERVICES[0]);
    let dir = s.root.path().join("bin");
    fs::create_dir_all(&dir).unwrap();
    let name = if cfg!(target_os = "windows") {
        "brave-browser.exe"
    } else {
        "brave-browser"
    };
    fs::copy(fixture(), dir.join(name)).unwrap();
    let file = s.root.path().join("browser.json");
    fs::write(&file, browser_scenario(&s).to_string()).unwrap();
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    s.command()
        .args(["login", s.service.group])
        .env_remove("COLMSG_BROWSER")
        .env("PATH", dir)
        .env("BROWSER_SCENARIO", file)
        .env("BROWSER_PROFILE", s.root.path().join("profile.txt"))
        .env("BROWSER_LOG", s.root.path().join("browser.log"))
        .assert()
        .success();
    refresh.assert();
    who.assert();
    assert_profile_retained(&s);
}
#[test]
fn unconfigured_services_report_missing_authentication() {
    let s = scenario(SERVICES[0]);
    for service in SERVICES {
        s.command().args(["-g", service.group]).assert().failure();
    }
}

fn process_command(s: &Scenario) -> std::process::Command {
    let mut c = std::process::Command::new(assert_cmd::cargo::cargo_bin("colmsg"));
    c.current_dir(s.root.path());
    for (key, value) in [
        ("HOME", s.root.path().join("home")),
        ("USERPROFILE", s.root.path().join("home")),
        ("XDG_CONFIG_HOME", s.root.path().join("config")),
        ("APPDATA", s.root.path().join("config")),
        ("COLMSG_CONFIG_DIR", s.root.path().join("config/colmsg")),
        ("COLMSG_CONFIG_PATH", s.config_file()),
    ] {
        c.env(key, value);
    }
    for service in SERVICES {
        c.env(service.base_env, s.server.url());
    }
    for key in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ] {
        c.env_remove(key);
    }
    c.env("NO_PROXY", "*").env("no_proxy", "*");
    c.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    c
}
fn bounded_output(mut child: std::process::Child) -> std::process::Output {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if std::time::Instant::now() > deadline {
            child.kill().unwrap();
            let out = child.wait_with_output().unwrap();
            panic!("CLI timed out: {}", String::from_utf8_lossy(&out.stderr));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
#[test]
fn concurrent_cli_runs_rotate_the_session_only_once() {
    let mut s = scenario(SERVICES[0]);
    write_credential(&s, &credential(&s, 0));
    let body =
        json!({"access_token":format!("test-access-token-{}",s.service.group),"expires_in":3600})
            .to_string();
    let refresh = s
        .server
        .mock("POST", "/v2/update_token")
        .match_header("cookie", "session=old-secret")
        .with_header("content-type", "application/json")
        .with_header(
            "set-cookie",
            &format!(
                "session=rotated-secret; Domain={}; Path=/v2/update_token; Secure",
                host(&s)
            ),
        )
        .with_chunked_body(move |writer| {
            std::thread::sleep(std::time::Duration::from_millis(300));
            writer.write_all(body.as_bytes())
        })
        .expect(1)
        .create();
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    let groups = s
        .server
        .mock("GET", "/v2/groups?")
        .match_header(
            "authorization",
            format!("Bearer test-access-token-{}", s.service.group).as_str(),
        )
        .with_header("content-type", "application/json")
        .with_body("[]")
        .expect(2)
        .create();
    let tags = s
        .server
        .mock("GET", "/v2/tags?")
        .with_header("content-type", "application/json")
        .with_body("[]")
        .expect(2)
        .create();
    s.set_global_members_repeated(json!([]), 2);
    let mut a = process_command(&s);
    a.args(["-g", s.service.group, "--dir"]).arg(s.output());
    let mut b = process_command(&s);
    b.args(["-g", s.service.group, "--dir"]).arg(s.output());
    let a = a.spawn().unwrap();
    let b = b.spawn().unwrap();
    for child in [a, b] {
        let out = bounded_output(child);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    refresh.assert();
    who.assert();
    groups.assert();
    tags.assert();
    s.assert_global_members_mock();
    assert!(saved(&s)["expires_at"].as_i64().unwrap() > chrono::Utc::now().timestamp());
}
#[cfg(unix)]
#[test]
fn cancelling_browser_login_cleans_up_during_startup_and_during_network_wait() {
    for startup in [true, false] {
        let s = scenario(SERVICES[0]);
        let c = credential(&s, 123);
        write_credential(&s, &c);
        let mut data = browser_scenario(&s);
        if startup {
            data["startup_wait"] = json!(true);
        } else {
            data["events"] = json!([]);
        }
        let file = s.root.path().join("browser.json");
        fs::write(&file, data.to_string()).unwrap();
        let mut command = process_command(&s);
        command
            .args(["login", s.service.group, "--browser"])
            .arg(fixture())
            .env("BROWSER_SCENARIO", file)
            .env("BROWSER_PROFILE", s.root.path().join("profile.txt"))
            .env("BROWSER_LOG", s.root.path().join("browser.log"));
        let mut child = command.spawn().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let marker = s.root.path().join(if startup {
            "profile.txt"
        } else {
            "browser.log"
        });
        while !marker.is_file()
            || (!startup
                && !fs::read_to_string(&marker)
                    .unwrap_or_default()
                    .contains("Page.navigate"))
        {
            if std::time::Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("browser did not start");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // Allow a read timeout before interrupting, as real users can wait between login steps.
        if !startup {
            std::thread::sleep(std::time::Duration::from_millis(1200));
        }
        assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGINT) }, 0);
        let out = bounded_output(child);
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("cancelled"));
        assert_eq!(saved(&s), c);
        assert_profile_retained(&s);
    }
}
#[test]
fn dropped_browser_connection_is_reported_and_cleans_up() {
    let s = scenario(SERVICES[0]);
    let mut data = browser_scenario(&s);
    data["disconnect"] = json!(true);
    login(&s, data).assert().failure();
    assert_profile_retained(&s);
    assert!(!auth_path(&s).exists());
}
#[test]
fn events_delivered_after_navigation_are_observed() {
    let mut s = scenario(SERVICES[0]);
    let mut data = browser_scenario(&s);
    data["events"]
        .as_array_mut()
        .unwrap()
        .insert(0, json!({"method":"Page.loadEventFired","params":{}}));
    data["delay_ms"] = json!(1100);
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    login(&s, data).assert().success();
    refresh.assert();
    who.assert();
    assert_profile_retained(&s);
}

#[test]
fn root_path_cookies_are_sent_only_to_their_canonical_origin() {
    let mut s = scenario(SERVICES[0]);
    let mut c = credential(&s, 0);
    c["cookies"].as_array_mut().unwrap().push(json!(format!(
        "other=root-cookie; Domain={}; Path=/; Secure; HttpOnly",
        host(&s)
    )));
    write_credential(&s, &c);
    let refresh = s.server.mock("POST","/v2/update_token").match_header("cookie",Matcher::AllOf(vec![Matcher::Regex("session=old-secret".into()),Matcher::Regex("other=root-cookie".into())])).with_header("content-type","application/json").with_body(json!({"access_token":format!("test-access-token-{}",s.service.group),"expires_in":3600}).to_string()).expect(1).create();
    let who = s
        .server
        .mock("GET", "/v2/account")
        .match_header("cookie", "other=root-cookie")
        .match_header(
            "authorization",
            format!("Bearer test-access-token-{}", s.service.group).as_str(),
        )
        .with_header("content-type", "application/json")
        .with_body("{\"username\":\"test-user\"}")
        .expect(1)
        .create();
    assert_download(&mut s);
    refresh.assert();
    who.assert();
}
#[test]
fn auth_status_distinguishes_all_unconfigured_and_native_services() {
    let s = scenario(SERVICES[0]);
    let out = s.command().args(["auth", "status"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    for service in SERVICES {
        assert!(text.contains(&format!("{}: not configured", service.group)));
    }
    fs::write(
        s.config_file(),
        SERVICES
            .iter()
            .map(|v| format!("{} native-secret", v.token_flag))
            .collect::<Vec<_>>()
            .join(" "),
    )
    .unwrap();
    let out = s.command().args(["auth", "status"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    for service in SERVICES {
        assert!(text.contains(&format!("{}: refresh-token", service.group)));
    }
    assert!(!text.contains("native-secret"));
}
#[test]
fn missing_cookies_and_headers_are_reported_without_saving_credentials() {
    for field in ["headers", "cookies"] {
        let s = scenario(SERVICES[0]);
        let mut data = browser_scenario(&s);
        if field == "headers" {
            data["events"][0]["params"]["request"]["headers"] = Value::Null;
        } else {
            data["cookies"] = Value::Null;
        }
        // Headerless requests are still observed; cookie failure occurs before any API access.
        data["cookies"] = Value::Null;
        login(&s, data).assert().failure();
        assert!(!auth_path(&s).exists());
        assert_profile_retained(&s);
    }
}
#[test]
fn save_failure_does_not_report_success_and_browser_is_cleaned_up() {
    let mut s = scenario(SERVICES[0]);
    fs::create_dir_all(auth_path(&s)).unwrap();
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    login(&s, browser_scenario(&s)).assert().failure();
    refresh.assert();
    who.assert();
    assert_profile_retained(&s);
    assert!(auth_path(&s).is_dir());
}
#[test]
fn invalid_api_url_configuration_and_lock_files_fail_safely() {
    let s = scenario(SERVICES[0]);
    write_credential(&s, &credential(&s, 0));
    download(&s)
        .env(s.service.base_env, "invalid URL")
        .assert()
        .failure();
    fs::remove_file(auth_path(&s).with_extension("lock")).unwrap();
    fs::create_dir_all(auth_path(&s).with_extension("lock")).unwrap();
    download(&s).assert().failure();
    assert_eq!(saved(&s)["expires_at"], 0);
}

#[test]
fn invalid_debugging_port_times_out_and_retains_browser_profile() {
    let s = scenario(SERVICES[0]);
    let mut data = browser_scenario(&s);
    data["invalid_port"] = json!(true);
    let out = login(&s, data)
        .timeout(std::time::Duration::from_secs(35))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("debugging endpoint did not start"));
    assert_profile_retained(&s);
    assert!(!auth_path(&s).exists());
}
#[test]
fn unavailable_debugging_socket_is_reported_without_saving_credentials() {
    let s = scenario(SERVICES[0]);
    let mut data = browser_scenario(&s);
    data["targets"] =
        json!([{"type":"page","webSocketDebuggerUrl":"ws://127.0.0.1:0/devtools/page/test"}]);
    let out = login(&s, data).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Could not connect to browser"));
    assert_profile_retained(&s);
}
#[cfg(unix)]
#[test]
fn authentication_directory_permissions_are_repaired_when_saving_rotated_credentials() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let mut s = scenario(SERVICES[0]);
    let c = credential(&s, 0);
    write_credential(&s, &c);
    fs::write(auth_path(&s).with_extension("lock"), "").unwrap();
    let refresh = good_update(&mut s);
    let who = account(&mut s, 200, json!({"username":"test-user"}));
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(INITIAL_DATE, 100, vec![]);
    let dir = auth_path(&s).parent().unwrap().to_owned();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();
    let out = download(&s).output().unwrap();
    let mode = fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(saved(&s)["cookies"][0]
        .as_str()
        .unwrap()
        .contains("rotated-secret"));
    assert_eq!(mode, 0o700);
    refresh.assert();
    who.assert();
    past.assert();
    page.assert();
    for mock in catalog {
        mock.assert();
    }
}
