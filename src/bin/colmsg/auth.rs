use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    env, fs,
    io::Write,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use colmsg::{controller::Service, dirs::PROJECT_DIRS, errors::*};
use reqwest::{
    blocking::Client,
    cookie::{CookieStore, Jar},
    header::{COOKIE, SET_COOKIE},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

static CANCELLED: AtomicBool = AtomicBool::new(false);

pub const SERVICES: &[&str] = &[
    "sakurazaka",
    "hinatazaka",
    "nogizaka",
    "asukasaito",
    "maishiraishi",
    "yodel",
];

pub fn service(slug: &str) -> Result<Service> {
    Service::ALL
        .iter()
        .copied()
        .find(|s| s.slug() == slug)
        .ok_or_else(|| format!("Unknown service: {}", slug).into())
}

fn web_host(service: Service) -> &'static str {
    match service {
        Service::Sakurazaka => "message.sakurazaka46.com",
        Service::Hinatazaka => "message.hinatazaka46.com",
        Service::Nogizaka => "message.nogizaka46.com",
        Service::Asukasaito => "message.asukasaito.jp",
        Service::Maishiraishi => "message.maishiraishi-official.com",
        Service::Yodel => "service.yodel-app.com",
    }
}

fn web_api_hosts(service: Service) -> [String; 3] {
    let host = web_host(service);
    [
        host.to_owned(),
        format!("api.{}", host),
        format!("api.{}", host.split_once('.').unwrap().1),
    ]
}

fn trusted_api(service: Service, url: &Url) -> bool {
    url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && url
            .host_str()
            .is_some_and(|h| web_api_hosts(service).iter().any(|host| host == h))
}

#[derive(Serialize, Deserialize)]
struct Credential {
    version: u32,
    api_url: String,
    headers: BTreeMap<String, String>,
    cookies: Vec<String>,
    username: String,
    access_token: String,
    expires_at: i64,
    cookie_expires_at: Option<i64>,
}

fn path(service: Service) -> PathBuf {
    PROJECT_DIRS
        .config_dir()
        .join("auth")
        .join(format!("{}.json", service.slug()))
}

pub fn has_browser_login(service: Service) -> bool {
    path(service).is_file()
}

fn load(service: Service) -> Result<Credential> {
    let bytes = fs::read(path(service))?;
    let c: Credential = serde_json::from_slice(&bytes)
        .map_err(|_| Error::Msg("Invalid authentication file".into()))?;
    let url = Url::parse(&c.api_url)?;
    if c.version != 1
        || !trusted_api(service, &url)
        || url.path() != "/v2/update_token"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Invalid authentication endpoint or format".into());
    }
    Ok(c)
}

fn save(service: Service, c: &Credential) -> Result<()> {
    let path = path(service);
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    let mut file = tempfile::NamedTempFile::new_in(dir)?;
    serde_json::to_writer(file.as_file_mut(), c)
        .map_err(|_| Error::Msg("Could not serialize authentication".into()))?;
    file.flush()?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}

fn web_client(c: &Credential) -> Result<(Client, Arc<Jar>, Url)> {
    let url = Url::parse(&c.api_url)?;
    let jar = Arc::new(Jar::default());
    for cookie in &c.cookies {
        jar.add_cookie_str(cookie, &url);
    }
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()?;
    Ok((client, jar, url))
}

fn transport_url(service: Service, url: &Url) -> Result<Url> {
    let key = match service {
        Service::Sakurazaka => "S_BASE_URL",
        Service::Hinatazaka => "H_BASE_URL",
        Service::Nogizaka => "N_BASE_URL",
        Service::Asukasaito => "A_BASE_URL",
        Service::Maishiraishi => "M_BASE_URL",
        Service::Yodel => "Y_BASE_URL",
    };
    match env::var(key) {
        Ok(base) => Ok(Url::parse(&base)?.join(url.path())?),
        Err(_) => Ok(url.clone()),
    }
}

fn with_headers(
    mut request: reqwest::blocking::RequestBuilder,
    c: &Credential,
) -> reqwest::blocking::RequestBuilder {
    for (name, value) in &c.headers {
        request = request.header(name, value);
    }
    request
}

fn update(service: Service, c: &mut Credential) -> Result<()> {
    let (client, jar, url) = web_client(c)?;
    let mut request = with_headers(client.post(transport_url(service, &url)?), c);
    if let Some(cookies) = jar.cookies(&url) {
        request = request.header(COOKIE, cookies);
    }
    let response = request.json(&json!({"refresh_token": null})).send()?;
    if !response.status().is_success() {
        return Err(format!(
            "{} authentication update failed (HTTP {}). Run: colmsg login {}",
            service.name(),
            response.status().as_u16(),
            service.slug()
        )
        .into());
    }
    let rotated = response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .map(|h| h.to_str().map(str::to_owned))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| Error::Msg("Invalid authentication cookie header".into()))?;
    for cookie in &rotated {
        jar.add_cookie_str(cookie, &url);
    }
    if !jar.cookies(&url).is_some_and(|cookies| {
        cookies.to_str().is_ok_and(|value| {
            value
                .split(';')
                .any(|pair| pair.trim().starts_with("session="))
        })
    }) {
        return Err("Authentication update removed the session cookie".into());
    }
    let mut cookies_updated = false;
    for cookie in rotated {
        // Keep cookies for this update endpoint only; logout-path cookies are unnecessary.
        let pair = cookie.split(';').next().unwrap_or("");
        let name = pair.split('=').next().unwrap_or("");
        if cookie
            .split(';')
            .any(|a| a.trim().eq_ignore_ascii_case("Path=/v2/signout"))
        {
            continue;
        }
        c.cookies
            .retain(|old| old.split(';').next().unwrap_or("").split('=').next() != Some(name));
        if let Some(expires) = cookie
            .split(';')
            .find_map(|a| a.trim().strip_prefix("Expires="))
        {
            if let Ok(date) = chrono::DateTime::parse_from_rfc2822(expires) {
                c.cookie_expires_at = Some(date.timestamp());
            }
        }
        c.cookies.push(cookie);
        cookies_updated = true;
    }
    if cookies_updated && !c.username.is_empty() {
        // A rotated cookie may invalidate the previous one even if the body is
        // malformed. Preserve it before parsing, without caching an unverified
        // token. New logins remain unsaved until their account is verified.
        c.access_token.clear();
        c.expires_at = 0;
        save(service, c)?;
    }
    let body: Value = response
        .json()
        .map_err(|_| Error::Msg("Invalid authentication update response".into()))?;
    c.access_token = body["access_token"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("Authentication response has no access token")?
        .to_owned();
    let seconds = body["expires_in"]
        .as_i64()
        .filter(|s| *s > 0)
        .ok_or("Authentication response has no token lifetime")?;
    c.expires_at = Utc::now()
        .timestamp()
        .checked_add(seconds)
        .ok_or("Invalid token lifetime")?;
    Ok(())
}

fn account(service: Service, c: &Credential) -> Result<String> {
    let (client, jar, url) = web_client(c)?;
    let account_url = url.join("/v2/account")?;
    let mut request = with_headers(client.get(transport_url(service, &account_url)?), c);
    if let Some(cookies) = jar.cookies(&account_url) {
        request = request.header(COOKIE, cookies);
    }
    let response = request.bearer_auth(&c.access_token).send()?;
    if !response.status().is_success() {
        return Err(format!(
            "Account verification failed (HTTP {})",
            response.status().as_u16()
        )
        .into());
    }
    let body: Value = response
        .json()
        .map_err(|_| Error::Msg("Invalid account response".into()))?;
    Ok(body["username"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("Account response has no username")?
        .to_owned())
}

pub fn endpoint(service: Service) -> Result<(String, reqwest::header::HeaderMap)> {
    let c = load(service)?;
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in &c.headers {
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| Error::Msg("Invalid authentication header name".into()))?;
        headers.insert(name, value.parse()?);
    }
    headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);
    Ok((
        transport_url(service, &Url::parse(&c.api_url)?.join("/")?)?.to_string(),
        headers,
    ))
}

pub fn token(service: Service, force: bool) -> Result<String> {
    // One worker per service; also protect against overlapping CLI processes.
    if !has_browser_login(service) {
        return Err(format!(
            "No browser login saved. Run: colmsg login {}",
            service.slug()
        )
        .into());
    }
    let lock_path = path(service).with_extension("lock");
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;
    lock.lock()?;
    let mut c = load(service)?;
    if force || c.expires_at <= Utc::now().timestamp() + 60 {
        update(service, &mut c)?;
        let expires_at = c.expires_at;
        c.expires_at = 0;
        save(service, &c)?;
        let username = account(service, &c)?;
        if username != c.username {
            return Err("Authentication account changed; log in again".into());
        }
        c.expires_at = expires_at;
        save(service, &c)?;
    }
    Ok(c.access_token)
}

fn expiry(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|d| d.to_rfc3339())
        .unwrap_or_else(|| "unknown".into())
}

pub fn status() -> Result<()> {
    let args = crate::config::get_args_from_config_file()?;
    for service in Service::ALL {
        if has_browser_login(service) {
            let c = load(service)?;
            println!(
                "{}: browser, account={}, access_expires={}, cookie_expires={}",
                service.slug(),
                c.username,
                expiry(c.expires_at),
                c.cookie_expires_at
                    .map(expiry)
                    .unwrap_or_else(|| "session/unknown".into())
            );
        } else {
            let prefix = match service {
                Service::Sakurazaka => "s",
                Service::Hinatazaka => "h",
                Service::Nogizaka => "n",
                Service::Asukasaito => "a",
                Service::Maishiraishi => "m",
                Service::Yodel => "y",
            };
            let option = format!("--{}_refresh_token", prefix);
            let configured = args.iter().any(|a| a == option.as_str());
            println!(
                "{}: {}",
                service.slug(),
                if configured {
                    "refresh-token (deprecated; configured; not verified)"
                } else {
                    "not configured"
                }
            );
        }
    }
    Ok(())
}

struct Browser {
    child: Child,
    cdp: Option<Cdp>,
}
impl Browser {
    fn close(&mut self) -> Result<()> {
        if let Some(mut cdp) = self.cdp.take() {
            // Close normally so Chromium flushes the persistent profile, including
            // identity-provider cookies. Cleanup also runs after cancellation.
            cdp.id += 1;
            let _ = cdp.socket.send(Message::Text(
                json!({"id":cdp.id,"method":"Browser.close","params":{}})
                    .to_string()
                    .into(),
            ));
            let _ = cdp.socket.close(None);
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.child.try_wait()?.is_none() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(100));
            }
        }
        if self.child.try_wait()?.is_none() {
            self.child.kill()?;
        }
        self.child.wait()?;
        Ok(())
    }
}
impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn executable(explicit: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.into());
    }
    if let Some(path) = env::var_os("COLMSG_BROWSER") {
        return Ok(path.into());
    }
    let mut candidates = Vec::new();
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            for name in [
                "brave-browser",
                "brave",
                "google-chrome",
                "chromium",
                "chromium-browser",
                "microsoft-edge",
            ] {
                let candidate = dir.join(name);
                #[cfg(target_os = "windows")]
                let candidate = candidate.with_extension("exe");
                candidates.push(candidate);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        candidates.extend(
            [
                "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            ]
            .map(PathBuf::from),
        );
    }
    #[cfg(target_os = "windows")]
    {
        for base in ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
            if let Some(base) = env::var_os(base) {
                for suffix in [
                    "BraveSoftware/Brave-Browser/Application/brave.exe",
                    "Google/Chrome/Application/chrome.exe",
                    "Microsoft/Edge/Application/msedge.exe",
                ] {
                    candidates.push(PathBuf::from(&base).join(suffix));
                }
            }
        }
    }
    candidates.into_iter().find(|p| p.is_file()).ok_or_else(|| {
        "No Chromium browser found. Specify --browser PATH or COLMSG_BROWSER.".into()
    })
}

struct Cdp {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    id: u64,
    events: VecDeque<Value>,
    deadline: Instant,
}
impl Cdp {
    fn read(&mut self) -> Result<Value> {
        loop {
            if CANCELLED.load(Ordering::Relaxed) {
                return Err("Browser login cancelled".into());
            }
            if Instant::now() >= self.deadline {
                return Err("Browser login timed out".into());
            }
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    return serde_json::from_str(&text)
                        .map_err(|_| "Invalid browser protocol response".into())
                }
                Ok(Message::Close(_)) => return Err("Browser closed before login completed".into()),
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => return Err("Browser connection closed before login completed".into()),
            }
        }
    }
    fn command(&mut self, method: &str, params: Value) -> Result<Value> {
        self.id += 1;
        let id = self.id;
        self.socket
            .send(Message::Text(
                json!({"id":id,"method":method,"params":params})
                    .to_string()
                    .into(),
            ))
            .map_err(|_| Error::Msg("Could not send browser command".into()))?;
        loop {
            let event = self.read()?;
            if event["id"].as_u64() == Some(id) {
                if !event["error"].is_null() {
                    return Err(format!("Browser command failed: {}", method).into());
                }
                return Ok(event["result"].clone());
            }
            if event["method"].is_string() {
                self.events.push_back(event);
            }
        }
    }
    fn event(&mut self) -> Result<Value> {
        if let Some(event) = self.events.pop_front() {
            Ok(event)
        } else {
            self.read()
        }
    }
}

pub fn login(service: Service, browser_path: Option<&str>) -> Result<()> {
    ctrlc::set_handler(|| CANCELLED.store(true, Ordering::Relaxed))
        .map_err(|_| Error::Msg("Could not register login cancellation handler".into()))?;
    let auth_path = path(service);
    fs::create_dir_all(auth_path.parent().unwrap())?;
    let login_lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(auth_path.with_extension("lock"))?;
    login_lock.try_lock().map_err(|_| {
        Error::Msg("Authentication is already in use by another colmsg process".into())
    })?;
    let executable = executable(browser_path)?;
    let profile_root = PROJECT_DIRS.config_dir().join("browser");
    fs::create_dir_all(&profile_root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&profile_root, fs::Permissions::from_mode(0o700))?;
    }
    // All services share the browser profile. Keep different browser executables
    // separate and serialize logins before touching any profile files.
    let profile_lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(profile_root.join("login.lock"))?;
    profile_lock.try_lock().map_err(|_| {
        Error::Msg("Browser login is already in use by another colmsg process".into())
    })?;
    let profile = profile_root.join(
        executable
            .file_name()
            .ok_or("Invalid browser executable path")?,
    );
    fs::create_dir_all(&profile)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&profile, fs::Permissions::from_mode(0o700))?;
    }
    let port_file = profile.join("DevToolsActivePort");
    // A previous browser run leaves this file behind; never connect to its port.
    match fs::remove_file(&port_file) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    // Port zero makes Chromium enable navigator.webdriver. Select a concrete
    // loopback port for this user-operated browser instead.
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let child = Command::new(executable)
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg(format!("--remote-debugging-port={}", port))
        .args([
            "--remote-debugging-address=127.0.0.1",
            "--no-first-run",
            "--no-default-browser-check",
            "--new-window",
            "about:blank",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let mut browser = Browser { child, cdp: None };
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .no_proxy()
        .build()?;
    let deadline = Instant::now() + Duration::from_secs(600);
    let startup = Instant::now() + Duration::from_secs(30);
    let targets: Value = loop {
        if CANCELLED.load(Ordering::Relaxed) {
            return Err("Browser login cancelled".into());
        }
        if browser.child.try_wait()?.is_some() {
            return Err("Browser exited during startup".into());
        }
        if let Ok(response) = client
            .get(format!("http://127.0.0.1:{}/json/list", port))
            .send()
        {
            break response.error_for_status()?.json()?;
        }
        if Instant::now() >= startup {
            return Err("Browser debugging endpoint did not start".into());
        }
        thread::sleep(Duration::from_millis(100));
    };
    let target = targets
        .as_array()
        .and_then(|a| a.iter().find(|t| t["type"] == "page"))
        .ok_or("Browser has no page target")?;
    let ws = target["webSocketDebuggerUrl"]
        .as_str()
        .ok_or("Browser has no debugging connection")?;
    let (mut socket, _) =
        tungstenite::connect(ws).map_err(|_| Error::Msg("Could not connect to browser".into()))?;
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;
    }
    browser.cdp = Some(Cdp {
        socket,
        id: 0,
        events: VecDeque::new(),
        deadline,
    });
    let cdp = browser.cdp.as_mut().unwrap();
    cdp.command(
        "Network.enable",
        json!({"maxTotalBufferSize": 10000000, "maxResourceBufferSize": 1000000}),
    )?;
    cdp.command("Page.enable", json!({}))?;
    // Authentication must never start a bulk message download.
    cdp.command(
        "Network.setBlockedURLs",
        json!({"urls": ["*/v2/groups/*/timeline*", "*/v2/groups/*/past_messages*"]}),
    )?;
    // Start a fresh service login on every invocation. Keep identity-provider
    // sessions (e.g. Google) and other services in the shared profile intact.
    for host in web_api_hosts(service) {
        cdp.command(
            "Storage.clearDataForOrigin",
            json!({"origin":format!("https://{}", host),"storageTypes":"all"}),
        )?;
    }
    cdp.command(
        "Page.navigate",
        json!({"url":format!("https://{}/", web_host(service))}),
    )?;
    println!(
        "{}: log in using your usual method in the browser. Waiting up to 10 minutes…",
        service.name()
    );
    let mut requests: BTreeMap<String, (Url, BTreeMap<String, String>)> = BTreeMap::new();
    let mut success = BTreeSet::new();
    loop {
        let event = cdp.event()?;
        let p = &event["params"];
        match event["method"].as_str() {
            Some("Network.requestWillBeSent") => {
                let request = &p["request"];
                if request["method"] != "POST" {
                    continue;
                }
                let Ok(url) = Url::parse(request["url"].as_str().unwrap_or("")) else {
                    continue;
                };
                if !trusted_api(service, &url)
                    || !matches!(url.path(), "/v2/signin" | "/v2/update_token")
                {
                    continue;
                }
                let mut headers = BTreeMap::new();
                if let Some(all) = request["headers"].as_object() {
                    for (name, value) in all {
                        if [
                            "x-talk-app-id",
                            "x-talk-app-platform",
                            "accept-language",
                            "origin",
                            "referer",
                            "user-agent",
                        ]
                        .contains(&name.to_ascii_lowercase().as_str())
                        {
                            if let Some(value) = value.as_str() {
                                headers.insert(name.to_owned(), value.to_owned());
                            }
                        }
                    }
                }
                headers.insert("Accept".into(), "application/json".into());
                requests.insert(
                    p["requestId"].as_str().unwrap_or("").to_owned(),
                    (url, headers),
                );
            }
            Some("Network.responseReceived") => {
                let id = p["requestId"].as_str().unwrap_or("");
                if p["response"]["status"].as_f64() == Some(200.0) && requests.contains_key(id) {
                    success.insert(id.to_owned());
                }
            }
            Some("Network.loadingFinished") => {
                let id = p["requestId"].as_str().unwrap_or("");
                if !success.remove(id) {
                    continue;
                }
                let (url, headers) = requests
                    .remove(id)
                    .ok_or("Missing authentication request")?;
                let result = cdp.command("Network.getResponseBody", json!({"requestId":id}))?;
                if result["base64Encoded"] == true {
                    return Err("Unexpected encoded authentication response".into());
                }
                let body: Value = serde_json::from_str(result["body"].as_str().unwrap_or(""))
                    .map_err(|_| Error::Msg("Invalid signin response".into()))?;
                if body["access_token"].as_str().is_none() {
                    continue;
                }
                let update_url = url.join("/v2/update_token")?;
                let cookie_result =
                    cdp.command("Network.getCookies", json!({"urls":[update_url.as_str()]}))?;
                let mut cookies = Vec::new();
                let mut cookie_expires_at = None;
                if let Some(all) = cookie_result["cookies"].as_array() {
                    for cookie in all.iter().filter(|c| c["name"] == "session") {
                        let name = cookie["name"].as_str().ok_or("Invalid session cookie")?;
                        let value = cookie["value"].as_str().ok_or("Invalid session cookie")?;
                        let domain = cookie["domain"].as_str().ok_or("Invalid cookie domain")?;
                        let path = cookie["path"].as_str().ok_or("Invalid cookie path")?;
                        cookies.push(format!(
                            "{}={}; Domain={}; Path={}; Secure; HttpOnly",
                            name, value, domain, path
                        ));
                        cookie_expires_at = cookie["expires"]
                            .as_f64()
                            .filter(|v| *v > 0.0)
                            .map(|v| v as i64);
                    }
                }
                if let Some(expires) =
                    cookie_expires_at.and_then(|t| chrono::DateTime::from_timestamp(t, 0))
                {
                    for cookie in &mut cookies {
                        cookie.push_str(&format!(
                            "; Expires={}; SameSite=Strict",
                            expires.format("%a, %d %b %Y %H:%M:%S GMT")
                        ));
                    }
                }
                if cookies.is_empty() {
                    return Err(
                        "Signin succeeded but no session cookie was issued for token updates"
                            .into(),
                    );
                }
                let mut credential = Credential {
                    version: 1,
                    api_url: update_url.to_string(),
                    headers,
                    cookies,
                    username: String::new(),
                    access_token: String::new(),
                    expires_at: 0,
                    cookie_expires_at,
                };
                // Stop the browser first so it cannot race with cookie rotation.
                browser.close()?;
                update(service, &mut credential)?;
                credential.username = account(service, &credential)?;
                let dir = path(service).parent().unwrap().to_path_buf();
                fs::create_dir_all(&dir)?;
                save(service, &credential)?;
                println!("{}: login complete.", service.name());
                return Ok(());
            }
            Some("Network.loadingFailed") => {
                let id = p["requestId"].as_str().unwrap_or("");
                requests.remove(id);
                success.remove(id);
            }
            _ => {}
        }
    }
}
