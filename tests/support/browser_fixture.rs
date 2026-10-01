// A separate process implementing the browser's public DevTools protocol.
use serde_json::{json, Value};
use std::{
    env, fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
};
use tungstenite::Message;

fn main() {
    let Some(profile) =
        env::args().find_map(|a| a.strip_prefix("--user-data-dir=").map(PathBuf::from))
    else {
        return;
    };
    let scenario: Value =
        serde_json::from_slice(&fs::read(env::var_os("BROWSER_SCENARIO").unwrap()).unwrap())
            .unwrap();
    let storage_path = profile.join("browser-storage.json");
    let mut storage: Value = fs::read(&storage_path)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
        .unwrap_or_else(|| {
            scenario
                .get("initial_storage")
                .cloned()
                .unwrap_or_else(|| json!({}))
        });
    fs::write(
        env::var_os("BROWSER_PROFILE").unwrap(),
        profile.to_string_lossy().as_bytes(),
    )
    .unwrap();
    if scenario["startup_exit"] == true {
        return;
    }
    if scenario["startup_wait"] == true {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    let port: u16 = env::args()
        .find_map(|a| {
            a.strip_prefix("--remote-debugging-port=")
                .and_then(|p| p.parse().ok())
        })
        .unwrap();
    let listener = TcpListener::bind((
        "127.0.0.1",
        if scenario["invalid_port"] == true {
            0
        } else {
            port
        },
    ))
    .unwrap();
    let port = listener.local_addr().unwrap().port();
    fs::write(
        profile.join("DevToolsActivePort"),
        if scenario["invalid_port"] == true {
            "not-a-port".to_owned()
        } else {
            format!("{}\n/devtools/browser/test", port)
        },
    )
    .unwrap();
    let (mut http, _) = listener.accept().unwrap();
    let mut buffer = [0; 4096];
    http.read(&mut buffer).unwrap();
    let targets = scenario.get("targets").cloned().unwrap_or_else(|| json!([{"type":"page","webSocketDebuggerUrl":format!("ws://127.0.0.1:{}/devtools/page/test",port)}]));
    let body = targets.to_string();
    write!(http,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
    drop(http);
    let (stream, _) = listener.accept().unwrap();
    let mut socket = tungstenite::accept(stream).unwrap();
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(env::var_os("BROWSER_LOG").unwrap())
        .unwrap();
    loop {
        let text = match socket.read() {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };
        let request: Value = serde_json::from_str(&text).unwrap();
        writeln!(log, "{}", request).unwrap();
        log.flush().unwrap();
        let method = request["method"].as_str().unwrap();
        if scenario["command_error"] == method {
            socket
                .send(Message::Text(
                    json!({"id":request["id"],"error":{"message":"SECRET browser error"}})
                        .to_string()
                        .into(),
                ))
                .unwrap();
            continue;
        }
        if method == "Storage.clearDataForOrigin" {
            assert_eq!(request["params"]["storageTypes"], "all");
            storage
                .as_object_mut()
                .unwrap()
                .remove(request["params"]["origin"].as_str().unwrap());
            fs::write(&storage_path, storage.to_string()).unwrap();
        }
        let result = match method {
            "Network.getResponseBody" if scenario["bodies"][request["params"]["requestId"].as_str().unwrap_or("")].is_object() => scenario["bodies"][request["params"]["requestId"].as_str().unwrap()].clone(),
            "Network.getResponseBody" => scenario.get("body_result").cloned().unwrap_or_else(||json!({"body":"{\"access_token\":\"browser-only-token\"}","base64Encoded":false})),
            "Network.getCookies" => json!({"cookies":scenario["cookies"]}),
            _ => json!({}),
        };
        // Deliver events while a command is pending, exercising protocol event queuing.
        if method == "Page.navigate" {
            let url = url::Url::parse(request["params"]["url"].as_str().unwrap()).unwrap();
            let origin = url.origin().ascii_serialization();
            fs::write(profile.join("before-navigation.json"), storage.to_string()).unwrap();
            if storage.get(&origin).is_some() {
                socket
                    .send(Message::Text(
                        json!({"id":request["id"],"error":{"message":"Service is already logged in"}})
                            .to_string()
                            .into(),
                    ))
                    .unwrap();
                continue;
            }
            storage[&origin] = json!({"session":"new-service-session"});
            fs::write(&storage_path, storage.to_string()).unwrap();
            if scenario["disconnect"] == true {
                return;
            }
            if let Some(delay) = scenario["delay_ms"].as_u64() {
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
            if scenario["ping"] == true {
                socket.send(Message::Ping(vec![1, 2].into())).unwrap();
                socket.send(Message::Binary(vec![3].into())).unwrap();
            }
            for event in scenario["events"].as_array().unwrap() {
                socket
                    .send(Message::Text(event.to_string().into()))
                    .unwrap();
            }
            if scenario["invalid_protocol"] == true {
                socket
                    .send(Message::Text("SECRET invalid json".into()))
                    .unwrap();
            }
            if scenario["close"] == true {
                socket.close(None).unwrap();
                socket.flush().unwrap();
                // Keep the connection alive until the client observes the close frame.
                let _ = socket.read();
                break;
            }
        }
        socket
            .send(Message::Text(
                json!({"id":request["id"],"result":result})
                    .to_string()
                    .into(),
            ))
            .unwrap();
    }
}
