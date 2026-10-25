//! A minimal HTTP/1.1 server on 127.0.0.1 for the download tests. The tests
//! never use the internet: every response is written by hand.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

pub struct Request {
    pub path: String,
    /// Header names in lower case.
    pub headers: HashMap<String, String>,
}

pub struct Server {
    pub port: u16,
    hits: Arc<AtomicUsize>,
}

impl Server {
    /// Answers every request with what `handler` returns: the raw bytes of a
    /// whole response (see [`response`]). The connection is closed after it.
    pub fn start(handler: impl Fn(&Request) -> Vec<u8> + Send + 'static) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                counter.fetch_add(1, Ordering::SeqCst);
                let mut raw = Vec::new();
                let mut buf = [0u8; 1024];
                while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
                    match stream.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => raw.extend_from_slice(&buf[..n]),
                    }
                }
                let text = String::from_utf8_lossy(&raw).into_owned();
                let mut lines = text.split("\r\n");
                let path = lines
                    .next()
                    .and_then(|l| l.split(' ').nth(1))
                    .unwrap_or("/")
                    .to_owned();
                let headers = lines
                    .filter_map(|l| l.split_once(':'))
                    .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_owned()))
                    .collect();
                let reply = handler(&Request { path, headers });
                let _ = stream.write_all(&reply);
                let _ = stream.flush();
            }
        });
        Server { port, hits }
    }

    /// Connections accepted so far.
    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }
}

/// A complete response with the right Content-Length.
pub fn response(status: &str, headers: &[(&str, &str)], body: &[u8]) -> Vec<u8> {
    let mut out = format!("HTTP/1.1 {status}\r\nConnection: close\r\n");
    for (k, v) in headers {
        out.push_str(&format!("{k}: {v}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    let mut bytes = out.into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

/// A response whose headers announce `announced` bytes while `body` is sent.
pub fn response_announcing(announced: usize, body: &[u8]) -> Vec<u8> {
    let head =
        format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {announced}\r\n\r\n");
    let mut bytes = head.into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

/// An invented table in the `PatternExportTab` layout: three 3x3x3 lines.
pub fn table() -> String {
    let row = |code: &str, n: &str, name: &str, moves: &str| {
        [
            code,
            n,
            "Kat",
            "Category",
            "Unter",
            "Subgroup",
            "Muster",
            name,
            moves,
            "",
            "",
            "",
            "",
            "",
            "nobody 2000",
            "",
            "",
        ]
        .join("\t")
    };
    [
        row("T1.01", "1", "Pair", "R2 U2"),
        row("T1.01", "2", "Pair", "R2 U2 D2 D2"),
        row("T1.02", "1", "Slice", "R2 L2"),
    ]
    .join("\n")
}
