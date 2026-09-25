mod common;

use common::*;
use serde_json::json;

const STAMP: &str = "20260923010203";
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

#[cfg(unix)]
fn run_with_terminal(
    scenario: &Scenario,
    extra_args: &[&str],
    width: u16,
) -> (std::process::ExitStatus, String) {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::{ffi::OsStrExt, process::ExitStatusExt},
        },
        time::Instant,
    };

    let root = scenario.root.path();
    let mut args = vec![
        CString::new("/usr/bin/env").unwrap(),
        CString::new(format!("HOME={}", root.join("home").display())).unwrap(),
        CString::new(format!("USERPROFILE={}", root.join("home").display())).unwrap(),
        CString::new(format!("XDG_CONFIG_HOME={}", root.join("config").display())).unwrap(),
        CString::new(format!("APPDATA={}", root.join("config").display())).unwrap(),
        CString::new(format!(
            "COLMSG_CONFIG_PATH={}",
            scenario.config_file().display()
        ))
        .unwrap(),
        CString::new(format!(
            "COLMSG_CONFIG_DIR={}",
            root.join("config/colmsg").display()
        ))
        .unwrap(),
        CString::new("NO_PROXY=*").unwrap(),
        CString::new("no_proxy=*").unwrap(),
        CString::new("HTTP_PROXY=").unwrap(),
        CString::new("HTTPS_PROXY=").unwrap(),
        CString::new("ALL_PROXY=").unwrap(),
        CString::new("http_proxy=").unwrap(),
        CString::new("https_proxy=").unwrap(),
        CString::new("all_proxy=").unwrap(),
        CString::new("TERM=xterm-256color").unwrap(),
    ];
    for service in &SERVICES {
        args.push(CString::new(format!("{}={}", service.base_env, scenario.server.url())).unwrap());
    }
    args.push(
        CString::new(
            assert_cmd::cargo::cargo_bin("colmsg")
                .as_os_str()
                .as_bytes(),
        )
        .unwrap(),
    );
    args.extend(
        [scenario.service.token_flag, "test-refresh-token", "--dir"]
            .iter()
            .map(|arg| CString::new(*arg).unwrap()),
    );
    args.push(CString::new(scenario.output().as_os_str().as_bytes()).unwrap());
    args.extend(extra_args.iter().map(|arg| CString::new(*arg).unwrap()));
    let mut argv = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
    argv.push(std::ptr::null());
    let env_program = CString::new("/usr/bin/env").unwrap();
    let working_directory = CString::new(root.as_os_str().as_bytes()).unwrap();

    let mut master_fd = -1;
    let mut window = libc::winsize {
        ws_row: 24,
        ws_col: width,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let child = unsafe {
        libc::forkpty(
            &mut master_fd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut window,
        )
    };
    assert!(child >= 0, "failed to create a pseudo-terminal");
    if child == 0 {
        unsafe {
            if libc::chdir(working_directory.as_ptr()) != 0 {
                libc::_exit(126);
            }
            libc::execv(env_program.as_ptr(), argv.as_ptr());
            libc::_exit(127);
        }
    }

    let mut master = unsafe { fs::File::from_raw_fd(master_fd) };
    let mut terminal_output = Vec::new();
    let started = Instant::now();
    loop {
        let remaining = Duration::from_secs(15).saturating_sub(started.elapsed());
        if remaining.is_zero() {
            unsafe {
                libc::kill(child, libc::SIGKILL);
                libc::waitpid(child, std::ptr::null_mut(), 0);
            }
            panic!("CLI did not finish while connected to a pseudo-terminal");
        }
        let mut descriptor = libc::pollfd {
            fd: master.as_raw_fd(),
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        };
        let timeout = remaining.as_millis().min(i32::MAX as u128) as i32;
        let ready = unsafe { libc::poll(&mut descriptor, 1, timeout) };
        if ready == 0 {
            continue;
        }
        if ready < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            panic!("failed to poll pseudo-terminal output: {}", error);
        }

        let mut buffer = [0; 4096];
        match master.read(&mut buffer) {
            Ok(0) => break,
            Ok(length) => terminal_output.extend_from_slice(&buffer[..length]),
            Err(error) if error.raw_os_error() == Some(libc::EIO) => break,
            Err(error) if error.raw_os_error() == Some(libc::EINTR) => continue,
            Err(error) => panic!("failed to read pseudo-terminal output: {}", error),
        }
    }

    let mut raw_status = 0;
    let waited = unsafe { libc::waitpid(child, &mut raw_status, 0) };
    assert_eq!(waited, child);
    (
        std::process::ExitStatus::from_raw(raw_status),
        String::from_utf8_lossy(&terminal_output).into_owned(),
    )
}

#[derive(Default)]
struct RequestConcurrency {
    active: AtomicUsize,
    peak: AtomicUsize,
}

fn delayed_member_list(
    concurrency: Arc<RequestConcurrency>,
) -> impl Fn(&mut dyn Write) -> std::io::Result<()> + Send + Sync + 'static {
    move |writer| {
        let active = concurrency.active.fetch_add(1, Ordering::SeqCst) + 1;
        concurrency.peak.fetch_max(active, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(300));
        let result = writer.write_all(b"[]");
        concurrency.active.fetch_sub(1, Ordering::SeqCst);
        result
    }
}

fn member_parallelism_peak(jobs: Option<usize>, member_count: u32) -> usize {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let groups = (1..=member_count)
        .map(|id| group(id, &format!("メンバー{id}"), true, &[]))
        .collect::<Vec<_>>();
    let group_list = s.get("/v2/groups", json!(groups));
    let tags = s.get("/v2/tags", json!([]));
    let concurrency = Arc::new(RequestConcurrency::default());

    s.set_group_members_with_chunked_body(delayed_member_list(Arc::clone(&concurrency)));
    let remaining_group_members = (2..=member_count)
        .map(|id| {
            s.group_members_with_chunked_body(id, delayed_member_list(Arc::clone(&concurrency)))
        })
        .collect::<Vec<_>>();
    let past = (1..=member_count)
        .map(|id| s.past_for(id, vec![]))
        .collect::<Vec<_>>();
    let timelines = (1..=member_count)
        .map(|id| s.timeline_for(id, INITIAL_DATE, 100, vec![]))
        .collect::<Vec<_>>();

    let mut command = s.download();
    if let Some(jobs) = jobs {
        command.args(["--jobs", &jobs.to_string()]);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr)
        .ends_with(&format!("{} members saved.\n", member_count)));

    auth.assert();
    group_list.assert();
    tags.assert();
    s.assert_member_mocks();
    for mock in remaining_group_members
        .iter()
        .chain(&past)
        .chain(&timelines)
    {
        mock.assert();
    }

    concurrency.peak.load(Ordering::SeqCst)
}

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
            "一期生/テストメンバー/1_0_20260923010203_unknown.txt",
            "一期生/テストメンバー/2_1_20260923010203_unknown.jpg",
            "一期生/テストメンバー/2_1_20260923010203_unknown.txt",
            "一期生/テストメンバー/3_2_20260923010203_unknown.mp4",
            "一期生/テストメンバー/4_3_20260923010203_unknown.mp4",
            "一期生/テストメンバー/5_4_20260923010203_unknown.txt",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>()
    );
    for name in &["1_0", "2_1", "5_4"] {
        assert_eq!(
            fs::read_to_string(
                s.member_dir()
                    .join(format!("{}_{}_unknown.txt", name, STAMP)),
            )
            .unwrap(),
            format!("{}\n", TEXT)
        );
    }
    for name in &[
        "2_1_20260923010203_unknown.jpg",
        "3_2_20260923010203_unknown.mp4",
        "4_3_20260923010203_unknown.mp4",
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
fn member_saves_use_the_default_and_configured_concurrency() {
    assert_eq!(member_parallelism_peak(None, 6), 4);
    assert_eq!(member_parallelism_peak(Some(2), 6), 2);
    // --jobs is a worker count, not a hard cap of four.
    assert_eq!(member_parallelism_peak(Some(8), 6), 6);
}

#[test]
fn selected_services_run_concurrently() {
    let mut sakurazaka = Scenario::new(SERVICES[0]);
    let mut hinatazaka = Scenario::new(SERVICES[1]);
    let sakurazaka_auth = sakurazaka.auth();
    let hinatazaka_auth = hinatazaka.auth();
    let sakurazaka_catalog = sakurazaka.catalog();
    let hinatazaka_catalog = hinatazaka.catalog();
    let concurrency = Arc::new(RequestConcurrency::default());

    sakurazaka.set_group_members_with_chunked_body(delayed_member_list(Arc::clone(&concurrency)));
    hinatazaka.set_group_members_with_chunked_body(delayed_member_list(Arc::clone(&concurrency)));
    let sakurazaka_past = sakurazaka.past(vec![]);
    let sakurazaka_timeline = sakurazaka.timeline(INITIAL_DATE, 100, vec![]);
    let hinatazaka_past = hinatazaka.past(vec![]);
    let hinatazaka_timeline = hinatazaka.timeline(INITIAL_DATE, 100, vec![]);

    let mut command = sakurazaka.download();
    command
        .args([SERVICES[1].token_flag, "test-refresh-token"])
        .env(SERVICES[1].base_env, hinatazaka.server.url());
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(concurrency.peak.load(Ordering::SeqCst), 2);
    assert!(String::from_utf8_lossy(&output.stderr).ends_with("2 members saved.\n"));

    sakurazaka_auth.assert();
    hinatazaka_auth.assert();
    for mock in sakurazaka_catalog.iter().chain(&hinatazaka_catalog).chain([
        &sakurazaka_past,
        &sakurazaka_timeline,
        &hinatazaka_past,
        &hinatazaka_timeline,
    ]) {
        mock.assert();
    }
    sakurazaka.assert_member_mocks();
    hinatazaka.assert_member_mocks();
}

#[test]
fn member_progress_and_completion_are_logged_with_a_final_count() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let timeline = s.timeline(INITIAL_DATE, 100, vec![]);

    let output = s.download().output().unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[Sakurazaka  1/ 1] downloading: テストメンバー"));
    assert!(stderr.contains("[Sakurazaka  1/ 1] done: テストメンバー"));
    assert!(stderr.ends_with("1 member saved.\n"));

    auth.assert();
    past.assert();
    timeline.assert();
    for mock in catalog {
        mock.assert();
    }
}

#[cfg(unix)]
#[test]
fn terminal_progress_redraws_member_rows_and_truncates_long_names() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let name = "長い名前".repeat(12);
    let groups = s.get("/v2/groups", json!([group(1, &name, true, &[])]));
    let tags = s.get("/v2/tags", json!([]));
    let past = s.past(vec![]);
    let timeline = s.timeline(INITIAL_DATE, 100, vec![]);

    let (status, terminal_output) = run_with_terminal(&s, &[], 48);
    assert!(status.success());
    auth.assert();
    groups.assert();
    tags.assert();
    s.assert_member_mocks();
    past.assert();
    timeline.assert();
    assert!(
        terminal_output.contains("Sakurazaka"),
        "unexpected terminal output: {:?}",
        terminal_output
    );
    assert!(terminal_output.contains("[ 1/ 1]"));
    assert!(terminal_output.contains("downloading"));
    assert!(terminal_output.contains("done"));
    assert!(terminal_output.contains("…"));
    assert!(terminal_output.contains("1 member saved."));
}

#[cfg(unix)]
#[test]
fn terminal_progress_animates_while_a_member_is_saving() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let concurrency = Arc::new(RequestConcurrency::default());
    s.set_group_members_with_chunked_body(delayed_member_list(concurrency));
    let past = s.past(vec![]);
    let timeline = s.timeline(INITIAL_DATE, 100, vec![]);

    let (status, terminal_output) = run_with_terminal(&s, &[], 80);
    assert!(status.success());
    assert!(terminal_output.contains("[ 1/ 1] | downloading"));
    assert!(terminal_output.contains("[ 1/ 1] / downloading"));
    assert!(terminal_output.contains("done"));
    assert!(terminal_output.contains("1 member saved."));

    auth.assert();
    s.assert_member_mocks();
    past.assert();
    timeline.assert();
    for mock in catalog {
        mock.assert();
    }
}

#[cfg(unix)]
#[test]
fn terminal_progress_omits_services_without_matching_members() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let (status, terminal_output) = run_with_terminal(&s, &["--name", "not-a-member"], 80);
    assert!(status.success());
    assert!(!terminal_output.contains("Sakurazaka"));
    assert!(terminal_output.contains("No matching members found."));

    auth.assert();
    s.assert_member_mocks();
    for mock in catalog {
        mock.assert();
    }
}

#[test]
fn failed_member_progress_is_reported_in_the_final_count() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(
        INITIAL_DATE,
        100,
        vec![message(1, "picture", &s.server.url())],
    );
    let failure = s
        .server
        .mock("GET", "/media/1")
        .with_status(403)
        .expect(1)
        .create();

    let output = s.download().output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[Sakurazaka  1/ 1] failed: テストメンバー"));
    assert!(stderr.ends_with("1 member failed; 1 service failed.\n"));
    assert!(files(&s.output()).is_empty());

    auth.assert();
    past.assert();
    page.assert();
    failure.assert();
    for mock in catalog {
        mock.assert();
    }
}

#[cfg(unix)]
#[test]
fn terminal_progress_marks_a_service_that_failed_to_save_a_member() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();
    let past = s.past(vec![]);
    let page = s.timeline(
        INITIAL_DATE,
        100,
        vec![message(1, "picture", &s.server.url())],
    );
    let failure = s
        .server
        .mock("GET", "/media/1")
        .with_status(403)
        .expect(1)
        .create();

    let (status, terminal_output) = run_with_terminal(&s, &[], 80);
    assert_eq!(status.code(), Some(1));
    assert!(terminal_output.contains("(failed)"));
    assert!(terminal_output.contains("failed"));
    assert!(terminal_output.contains("1 member failed; 1 service failed."));

    auth.assert();
    past.assert();
    page.assert();
    failure.assert();
    for mock in catalog {
        mock.assert();
    }
}

#[cfg(unix)]
#[test]
fn terminal_progress_shows_retrying_after_a_member_request_expires() {
    let mut s = Scenario::new(SERVICES[0]);
    let initial_auth = s.auth();
    let retry_auth = s.auth();
    s.set_global_members_repeated(json!([]), 2);
    let groups_body = json!([
        group(1, "テストメンバー", true, &[]),
        group(2, "次のメンバー", true, &[])
    ]);
    let initial_groups = s.get("/v2/groups", groups_body.clone());
    let retry_groups = s.get("/v2/groups", groups_body);
    let initial_tags = s.get("/v2/tags", json!([]));
    let retry_tags = s.get("/v2/tags", json!([]));
    let expired_member = s.fail_group_members(401);
    let retried_member = s.get("/v2/groups/1/members", json!([]));
    let next_member = s.get("/v2/groups/2/members", json!([]));
    let past_one = s.past_for(1, vec![]);
    let past_two = s.past_for(2, vec![]);
    let timeline_one = s.timeline_for(1, INITIAL_DATE, 100, vec![]);
    let timeline_two = s.timeline_for(2, INITIAL_DATE, 100, vec![]);

    let (status, terminal_output) = run_with_terminal(&s, &["--jobs", "1"], 80);
    assert!(status.success());
    assert!(terminal_output.contains("(retrying)"));
    assert!(terminal_output.contains("2 members saved."));

    initial_auth.assert();
    retry_auth.assert();
    s.assert_global_members_mock();
    for mock in [
        &initial_groups,
        &retry_groups,
        &initial_tags,
        &retry_tags,
        &expired_member,
        &retried_member,
        &next_member,
        &past_one,
        &past_two,
        &timeline_one,
        &timeline_two,
    ] {
        mock.assert();
    }
}

#[test]
fn no_matching_members_only_prints_the_result_line() {
    let mut s = Scenario::new(SERVICES[0]);
    let auth = s.auth();
    let catalog = s.catalog();

    let output = s
        .download()
        .args(["--name", "not-a-member"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "No matching members found.\n"
    );

    auth.assert();
    for mock in catalog {
        mock.assert();
    }
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
    let help = s.command().arg("--help").output().unwrap();
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("--jobs"));
    assert!(help.contains("default: 4"));
    let output = s.command().arg("--config-path").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        PathBuf::from(String::from_utf8(output.stdout).unwrap().trim()),
        s.config_file()
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
        vec!["--jobs", "0"],
        vec!["--jobs", "not-a-number"],
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
    s.set_global_members(json!([]));
    s.set_group_members(json!([]));
    let past = s.past(vec![]);
    let page = s.timeline(INITIAL_DATE, 100, vec![message(1, "text", &s.server.url())]);
    s.download()
        .args(["--name", " テストメンバー "])
        .assert()
        .success();
    assert_eq!(
        files(&s.output()),
        vec![PathBuf::from(
            "テストメンバー/1_0_20260923010203_unknown.txt"
        )]
    );
    auth.assert();
    groups.assert();
    tags.assert();
    s.assert_member_mocks();
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
        fs::read_to_string(s.member_dir().join("1_0_20260923010203_unknown.txt")).unwrap(),
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
            .mock("GET", "/v2/members?")
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
    fs::create_dir_all(s.member_dir().join("1_0_20260923010203_unknown.txt")).unwrap();
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
    fs::write(
        s.member_dir().join("1_0_20261399010203_unknown.txt"),
        "keep",
    )
    .unwrap();
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
        "/v2/members?",
        "/v2/tags?",
        "/v2/groups/1/members?",
        "/v2/groups/1/past_messages?order=asc",
        "/v2/groups/1/timeline",
    ] {
        let mut s = Scenario::new(SERVICES[0]);
        let auth = s.auth();
        let mut mocks = vec![];
        if *path != "/v2/members?" {
            mocks.push(s.get(
                "/v2/groups",
                json!([group(1, "テスト メンバー", true, &["generation"])]),
            ));
        }
        if *path != "/v2/tags?" && *path != "/v2/members?" {
            mocks.push(s.get(
                "/v2/tags",
                json!([tag("generation", "一期生", json!(null))]),
            ));
        }
        if *path == "/v2/groups/1/timeline" {
            mocks.push(s.past(vec![]));
        }
        let failure = match *path {
            "/v2/members?" => s.fail_global_members(503),
            "/v2/groups/1/members?" => s.fail_group_members(503),
            _ => s
                .server
                .mock("GET", path.split('?').next().unwrap())
                .match_query(mockito::Matcher::Any)
                .with_status(503)
                .expect(1)
                .create(),
        };
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
        fs::read_to_string(s.member_dir().join("1_0_20260923010203_unknown.txt")).unwrap(),
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
        fs::read(s.member_dir().join("1_1_20260923010203_unknown.jpg")).unwrap(),
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
        fs::read(s.member_dir().join("1_0_20260923010203_unknown.txt")).unwrap(),
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
fn config_path_falls_back_to_isolated_home_when_xdg_is_unset_or_relative() {
    let s = Scenario::new(SERVICES[0]);
    for relative in [false, true].iter() {
        let mut command = s.command();
        command.env_remove("COLMSG_CONFIG_DIR");
        command.env_remove("COLMSG_CONFIG_PATH");
        command.env_remove("XDG_CONFIG_HOME");
        if *relative {
            command.env("XDG_CONFIG_HOME", "relative/config");
        }
        let output = command.arg("--config-path").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            PathBuf::from(String::from_utf8(output.stdout).unwrap().trim()),
            s.root.path().join("home/.config/colmsg/config")
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
        fs::read_to_string(s.member_dir().join("1_0_20260923010203_unknown.txt")).unwrap(),
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
    // If the response did not time out, the command can finish successfully after 31 seconds.
    let _tags = s.get("/v2/tags", json!([]));
    let output = s
        .download()
        .timeout(std::time::Duration::from_secs(40))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Sakurazaka:"),
        "unexpected stderr: {}",
        stderr
    );
    assert!(files(&s.output()).is_empty());
    auth.assert();
    stalled.assert();
}
