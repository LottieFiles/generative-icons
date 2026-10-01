//! Sign-in, status and sign-out against the mock engine.

mod common;

use std::time::Duration;

use common::{FakeBrowser, MockEngine};
use gai_engine::{Error, LoginMode, LoginOptions, login, logout, status};

fn options(mode: LoginMode) -> LoginOptions {
    LoginOptions {
        mode,
        open_browser: false,
        timeout: Duration::from_secs(20),
    }
}

#[test]
fn loopback_login_then_status_and_logout() {
    let engine = MockEngine::start("{}");
    let home = tempfile::tempdir().unwrap();
    let config = engine.config(home.path());
    assert_eq!(status(&config).unwrap(), None);

    let mut browser = FakeBrowser::new(engine.state.clone(), false);
    let signed_in = login(&config, &options(LoginMode::Loopback), &mut browser).unwrap();
    assert_eq!(signed_in.account.as_deref(), Some("designer@example.com"));
    assert_eq!(signed_in.client_id.as_deref(), Some("client-1"));
    assert_eq!(signed_in.credential_store, "file");
    assert_eq!(status(&config).unwrap().as_ref(), Some(&signed_in));

    {
        let state = engine.state.lock();
        let registration = &state.registrations[0];
        let redirect = registration["redirect_uris"][0].as_str().unwrap();
        assert!(
            redirect.starts_with("http://127.0.0.1:") && redirect.ends_with("/callback"),
            "{redirect}"
        );
        assert_eq!(registration["token_endpoint_auth_method"], "none");
        assert_eq!(
            registration["grant_types"],
            serde_json::json!(["authorization_code", "refresh_token"])
        );
        let shown = url::Url::parse(&browser.shown[0]).unwrap();
        assert!(
            shown
                .as_str()
                .starts_with(&format!("{}/authorize?", state.base))
        );
        assert!(
            shown
                .query_pairs()
                .any(|(k, v)| k == "resource" && v == format!("{}/mcp", state.base))
        );
    }

    let outcome = logout(&config).unwrap();
    assert!(outcome.was_signed_in && outcome.revoked);
    assert_eq!(engine.state.lock().revoked, ["rt-1"]);
    assert_eq!(status(&config).unwrap(), None);
    let again = logout(&config).unwrap();
    assert!(!again.was_signed_in && !again.revoked);
}

#[test]
fn login_reuses_the_client_and_registers_again_when_it_expired() {
    let engine = MockEngine::start("{}");
    let home = tempfile::tempdir().unwrap();
    let config = engine.config(home.path());
    let mut browser = FakeBrowser::new(engine.state.clone(), false);

    login(&config, &options(LoginMode::Loopback), &mut browser).unwrap();
    let second = login(&config, &options(LoginMode::Loopback), &mut browser).unwrap();
    assert_eq!(second.client_id.as_deref(), Some("client-1"));
    assert_eq!(engine.state.lock().registrations.len(), 1);
    // Signing in again revokes the session it replaces.
    assert_eq!(engine.state.lock().revoked, ["rt-1"]);

    // The server forgot client-1: the code exchange fails with invalid_client, the CLI
    // registers again and runs the consent once more.
    engine
        .state
        .lock()
        .invalid_clients
        .insert("client-1".into());
    let third = login(&config, &options(LoginMode::Loopback), &mut browser).unwrap();
    assert_eq!(third.client_id.as_deref(), Some("client-2"));
    assert_eq!(engine.state.lock().registrations.len(), 2);
    assert_eq!(browser.shown.len(), 4);
    assert_eq!(engine.state.lock().revoked, ["rt-1", "rt-2"]);
}

#[test]
fn paste_mode_login() {
    let engine = MockEngine::start("{}");
    let home = tempfile::tempdir().unwrap();
    let config = engine.config(home.path());
    let mut browser = FakeBrowser::new(engine.state.clone(), true);
    let signed_in = login(&config, &options(LoginMode::Paste), &mut browser).unwrap();
    assert_eq!(signed_in.client_id.as_deref(), Some("client-1"));
}

#[test]
fn a_denied_consent_cancels_the_login() {
    struct Denied;
    impl gai_engine::LoginInteraction for Denied {
        fn show_authorization_url(&mut self, _: &str, _: bool) {}
        fn read_redirect_url(&mut self) -> std::io::Result<String> {
            Ok("http://127.0.0.1:1/callback?error=access_denied&state=unknown".into())
        }
    }
    let engine = MockEngine::start("{}");
    let home = tempfile::tempdir().unwrap();
    let config = engine.config(home.path());
    // The state does not match this attempt, so the pasted address is refused outright.
    let err = login(&config, &options(LoginMode::Paste), &mut Denied).unwrap_err();
    assert!(matches!(err, Error::LoginCancelled(_)), "{err}");
    assert_eq!(err.code(), "LOGIN_CANCELLED");
    assert_eq!(status(&config).unwrap(), None);
}
