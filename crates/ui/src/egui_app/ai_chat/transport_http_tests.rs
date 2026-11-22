//! PR #2122 live-transport pins through the real turn worker and shutdown path.
//! Provenance: our own loopback fixtures; no external endpoint or key.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use camcraft_app::ai::Endpoint;
use serde_json::{Value, json};

use super::transport::{ChatTransport, TransportOutput, TransportRequest, default_presets};
use super::transport_http::HttpTransport;
use super::worker::{TurnInput, WorkerEvent, spawn_turn};

fn read_request(s: &mut TcpStream) {
    s.set_read_timeout(Some(Duration::from_secs(5)))
        .expect("request timeout");
    let mut received = Vec::new();
    let mut chunk = [0; 4096];
    loop {
        let count = s.read(&mut chunk).expect("request bytes");
        assert!(count > 0, "request ended early");
        received.extend_from_slice(&chunk[..count]);
        if let Some(end) = received.windows(4).position(|w| w == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&received[..end]);
            let length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().expect("body length"))
                })
                .unwrap_or(0);
            if received.len() >= end + 4 + length {
                return;
            }
        }
    }
}

struct LoopbackTransport(Endpoint);

impl ChatTransport for LoopbackTransport {
    fn complete(
        &self,
        req: &TransportRequest,
        cancel: &AtomicBool,
    ) -> Result<TransportOutput, String> {
        HttpTransport.complete_at(req, cancel, &self.0, None)
    }
}

fn listener() -> (TcpListener, Endpoint) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback");
    let ep = Endpoint::parse(&format!(
        "http://{}/v1",
        listener.local_addr().expect("addr")
    ))
    .expect("endpoint");
    (listener, ep)
}

#[test]
fn http_prompt_json_reply_reaches_worker_tool_dispatch() {
    let (listener, ep) = listener();
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().expect("accept");
        read_request(&mut s);
        let proposal = json!({"tool":"get_job", "arguments":{}}).to_string();
        let body = json!({"choices":[{"message":{"content":proposal}}]}).to_string();
        write!(
            s,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .expect("reply");
    });
    let preset = default_presets()
        .into_iter()
        .find(|p| p.id == "local-prompt-json")
        .expect("preset");
    let (mcp_tx, mcp_rx) = mpsc::channel();
    let worker = spawn_turn(TurnInput {
        turn_id: 1,
        model: "test-model".into(),
        preset,
        offline_only: true,
        initial_messages: vec![],
        project_snapshot: Value::Null,
        transport: Arc::new(LoopbackTransport(ep)),
        mcp_tx,
        wake: None,
    });
    let request = mcp_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("HTTP proposal must dispatch get_job through worker");
    let rpc: Value = serde_json::from_str(request.line()).expect("JSON-RPC");
    assert_eq!(rpc["params"]["name"], "get_job");
    assert_eq!(rpc["params"]["arguments"], json!({}));
    assert!(worker.rx.try_iter().any(
        |event| matches!(event, WorkerEvent::ToolCardAdded { card, .. } if card.name == "get_job")
    ));
    drop(worker);
    server.join().expect("server");
}

#[test]
fn cancel_interrupts_worker_waiting_for_http_reply() {
    assert_silent_server_stops(true);
}

#[test]
fn shutdown_interrupts_worker_waiting_for_http_reply() {
    assert_silent_server_stops(false);
}

fn assert_silent_server_stops(cancel_first: bool) {
    let (listener, ep) = listener();
    let (started_tx, started_rx) = mpsc::channel();
    let (closed_tx, closed_rx) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().expect("accept");
        s.set_read_timeout(Some(Duration::from_secs(5)))
            .expect("timeout");
        let mut buf = [0; 8192];
        read_request(&mut s);
        started_tx.send(()).expect("started");
        loop {
            match s.read(&mut buf) {
                Ok(0) => {
                    let _ = closed_tx.send(());
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
    let mut state = crate::state::State::new();
    let index = state
        .ai_chat
        .presets()
        .iter()
        .position(|p| p.id == "local-readonly")
        .expect("preset");
    state.ai_chat.select_preset(index);
    state.ai_chat.set_transport(Arc::new(LoopbackTransport(ep)));
    state.ai_chat.prompt_draft = "Hello".into();
    state.submit_ai_prompt();
    started_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("worker reached server");
    let start = Instant::now();
    if cancel_first {
        state.ai_chat.cancel();
        while state.ai_chat.is_running() && start.elapsed() < Duration::from_secs(1) {
            state.ai_chat.poll(&mut state.history);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            !state.ai_chat.is_running(),
            "Cancel must finish the active HTTP turn promptly"
        );
    } else {
        state.ai_chat.shutdown();
    }
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "cancel={cancel_first}: shutdown blocked for {:?}",
        start.elapsed()
    );
    assert!(!state.ai_chat.is_running());
    closed_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("shutdown must close active socket");
    server.join().expect("server");
}
