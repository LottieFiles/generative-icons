//! The loopback redirect target of the sign-in flow (RFC 8252 §7.3) and the parser for a
//! redirect URL the user pastes when the browser runs on another machine.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use url::Url;

use crate::error::{Error, Result};

/// The redirect path registered with the server; the server matches it exactly.
pub(crate) const CALLBACK_PATH: &str = "/callback";
/// The longest request head (request line and headers) read from one connection.
const MAX_HEAD: usize = 8 * 1024;
/// How long one connection may take to send its request head.
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// What the redirect carried.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Callback {
    /// The authorization code, with the state already checked.
    Code(String),
    /// A redirect for another sign-in attempt (stale tab or forged request); keep waiting.
    WrongState,
}

/// A one-shot HTTP listener on `127.0.0.1:<ephemeral port>`.
pub(crate) struct CallbackListener {
    listener: TcpListener,
    redirect_uri: String,
}

impl CallbackListener {
    pub fn bind() -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        Ok(CallbackListener {
            listener,
            redirect_uri: format!("http://127.0.0.1:{port}{CALLBACK_PATH}"),
        })
    }

    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    /// Serves requests until one carries a code for `state`, the server reports an error, or
    /// `timeout` passes. Each connection gets at most [`CONNECTION_TIMEOUT`] and never runs
    /// past the deadline, so a stalled client cannot hold the sign-in open.
    pub fn wait(&self, state: &str, timeout: Duration) -> Result<String> {
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                return Err(Error::LoginCancelled(format!(
                    "no sign-in within {} s",
                    timeout.as_secs()
                )));
            }
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let until = deadline.min(Instant::now() + CONNECTION_TIMEOUT);
                    if let Some(code) = self.serve(stream, state, until)? {
                        return Ok(code);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    /// Answers one request; returns the code once the right redirect arrives.
    fn serve(&self, mut stream: TcpStream, state: &str, until: Instant) -> Result<Option<String>> {
        stream.set_nonblocking(false)?;
        let Some(head) = read_head(&mut stream, until) else {
            return Ok(None);
        };
        let request_line = head.lines().next().unwrap_or_default();
        let target = request_line.split_whitespace().nth(1).unwrap_or("/");
        let url = Url::parse(&self.redirect_uri).and_then(|base| base.join(target));
        let outcome = match url {
            Ok(url) if url.path() == CALLBACK_PATH => parse_callback_url(&url, state),
            _ => {
                respond(&mut stream, "404 Not Found", "Not found.");
                return Ok(None);
            }
        };
        match outcome {
            Ok(Callback::Code(code)) => {
                respond(
                    &mut stream,
                    "200 OK",
                    "Signed in to generative-icons. You can close this tab.",
                );
                Ok(Some(code))
            }
            Ok(Callback::WrongState) => {
                respond(
                    &mut stream,
                    "400 Bad Request",
                    "This sign-in link is out of date. Start again from the terminal.",
                );
                Ok(None)
            }
            Err(e) => {
                respond(
                    &mut stream,
                    "400 Bad Request",
                    &format!("Sign-in failed: {e}"),
                );
                Err(e)
            }
        }
    }
}

/// The request head (up to the blank line), read by `until`; `None` when the client is too
/// slow, sends more than [`MAX_HEAD`] bytes or closes early. The whole head is read so the
/// browser gets the response rather than a reset.
fn read_head(stream: &mut TcpStream, until: Instant) -> Option<String> {
    let mut head = Vec::new();
    let mut buf = [0u8; 1024];
    let complete = |head: &[u8]| {
        head.windows(4).any(|w| w == b"\r\n\r\n") || head.windows(2).any(|w| w == b"\n\n")
    };
    while !complete(&head) {
        let left = until.checked_duration_since(Instant::now())?;
        if left.is_zero() {
            return None;
        }
        stream.set_read_timeout(Some(left)).ok()?;
        let n = stream.read(&mut buf).ok().filter(|&n| n > 0)?;
        head.extend_from_slice(&buf[..n]);
        if head.len() > MAX_HEAD {
            return None;
        }
    }
    Some(String::from_utf8_lossy(&head).into_owned())
}

fn respond(stream: &mut TcpStream, status: &str, message: &str) {
    let body = format!(
        "<!doctype html><meta charset=utf-8><title>generative-icons</title><p style=\"font:16px system-ui;margin:3em\">{}</p>",
        message.replace('&', "&amp;").replace('<', "&lt;")
    );
    // The browser may already have gone; nothing to do about a failed write.
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{body}",
        body.len()
    );
}

/// Reads `code` from a redirect URL, checking `state` and surfacing an `error` redirect.
pub(crate) fn parse_callback_url(url: &Url, state: &str) -> Result<Callback> {
    let mut code = None;
    let mut got_state = None;
    let mut error = None;
    let mut description = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => got_state = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            "error_description" => description = Some(value.into_owned()),
            _ => {}
        }
    }
    if got_state.as_deref() != Some(state) {
        return Ok(Callback::WrongState);
    }
    if let Some(error) = error {
        return Err(match error.as_str() {
            "access_denied" => {
                Error::LoginCancelled("access was denied on the consent page".into())
            }
            _ => Error::OAuth {
                endpoint: "authorization".into(),
                error,
                description,
            },
        });
    }
    code.map(Callback::Code)
        .ok_or_else(|| Error::Protocol("the redirect carried neither a code nor an error".into()))
}

/// Reads the code from a redirect URL the user pasted; the state must match this attempt.
pub(crate) fn parse_pasted_redirect(pasted: &str, state: &str) -> Result<String> {
    let pasted = pasted.trim();
    let url = Url::parse(pasted).map_err(|_| {
        Error::LoginCancelled(format!(
            "{pasted:?} is not the address the browser was sent to"
        ))
    })?;
    match parse_callback_url(&url, state)? {
        Callback::Code(code) => Ok(code),
        Callback::WrongState => Err(Error::LoginCancelled(
            "the pasted address belongs to a different sign-in attempt".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_url_parsing() {
        let url = Url::parse("http://127.0.0.1:5/callback?code=abc&state=s1").unwrap();
        assert_eq!(
            parse_callback_url(&url, "s1").unwrap(),
            Callback::Code("abc".into())
        );
        assert_eq!(
            parse_callback_url(&url, "s2").unwrap(),
            Callback::WrongState
        );
        let denied =
            Url::parse("http://127.0.0.1:5/callback?error=access_denied&state=s1").unwrap();
        assert!(matches!(
            parse_callback_url(&denied, "s1"),
            Err(Error::LoginCancelled(_))
        ));
        let other = Url::parse(
            "http://127.0.0.1:5/callback?error=server_error&error_description=x&state=s1",
        )
        .unwrap();
        assert!(matches!(
            parse_callback_url(&other, "s1"),
            Err(Error::OAuth { .. })
        ));
    }

    #[test]
    fn pasted_redirects() {
        assert_eq!(
            parse_pasted_redirect(" http://127.0.0.1:5/callback?state=s&code=c%2B1\n", "s")
                .unwrap(),
            "c+1"
        );
        assert!(parse_pasted_redirect("http://127.0.0.1:5/callback?state=x&code=c", "s").is_err());
        assert!(parse_pasted_redirect("code=c", "s").is_err());
    }

    #[test]
    fn listener_ignores_strays_and_returns_the_code() {
        let listener = CallbackListener::bind().unwrap();
        let base = listener.redirect_uri().to_owned();
        let browser = thread::spawn(move || {
            let get = |target: &str| {
                let addr = base
                    .trim_start_matches("http://")
                    .split('/')
                    .next()
                    .unwrap()
                    .to_owned();
                let mut stream = TcpStream::connect(addr).unwrap();
                write!(stream, "GET {target} HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
                let mut response = String::new();
                std::io::Read::read_to_string(&mut stream, &mut response).unwrap();
                response
            };
            let favicon = get("/favicon.ico");
            let stale = get("/callback?code=old&state=other");
            let done = get("/callback?code=fresh&state=mine");
            (favicon, stale, done)
        });
        let code = listener.wait("mine", Duration::from_secs(10)).unwrap();
        assert_eq!(code, "fresh");
        let (favicon, stale, done) = browser.join().unwrap();
        assert!(favicon.starts_with("HTTP/1.1 404"));
        assert!(stale.starts_with("HTTP/1.1 400"));
        assert!(done.starts_with("HTTP/1.1 200") && done.contains("Signed in"));
    }

    fn connect(listener: &CallbackListener) -> TcpStream {
        let addr = listener
            .redirect_uri()
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap()
            .to_owned();
        TcpStream::connect(addr).unwrap()
    }

    #[test]
    fn a_trickling_client_cannot_outlast_the_timeout() {
        let listener = CallbackListener::bind().unwrap();
        let mut slow = connect(&listener);
        let trickle = thread::spawn(move || {
            // One byte at a time, each well inside the per-read timeout.
            for _ in 0..40 {
                if slow.write_all(b"G").is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
        let started = Instant::now();
        let err = listener.wait("s", Duration::from_secs(1)).unwrap_err();
        assert!(matches!(err, Error::LoginCancelled(_)), "{err}");
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "{:?}",
            started.elapsed()
        );
        trickle.join().unwrap();
    }

    #[test]
    fn an_oversized_request_is_dropped_and_the_redirect_still_served() {
        let listener = CallbackListener::bind().unwrap();
        let mut huge = connect(&listener);
        let mut good = connect(&listener);
        let browser = thread::spawn(move || {
            let header = format!("X-Pad: {}\r\n", "a".repeat(1000));
            let _ = write!(huge, "GET /callback?code=bad&state=mine HTTP/1.1\r\n");
            for _ in 0..20 {
                if huge.write_all(header.as_bytes()).is_err() {
                    break;
                }
            }
            write!(
                good,
                "GET /callback?code=fresh&state=mine HTTP/1.1\r\nHost: x\r\n\r\n"
            )
            .unwrap();
            let mut response = String::new();
            std::io::Read::read_to_string(&mut good, &mut response).unwrap();
            response
        });
        let code = listener.wait("mine", Duration::from_secs(10)).unwrap();
        assert_eq!(code, "fresh");
        assert!(browser.join().unwrap().starts_with("HTTP/1.1 200"));
    }

    #[test]
    fn listener_times_out() {
        let listener = CallbackListener::bind().unwrap();
        let err = listener.wait("s", Duration::from_millis(100)).unwrap_err();
        assert!(matches!(err, Error::LoginCancelled(_)));
    }
}
