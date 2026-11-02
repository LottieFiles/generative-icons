//! OAuth 2.1 for MCP: discovery (RFC 9728, RFC 8414), dynamic client registration (RFC 7591),
//! PKCE (RFC 7636), resource indicators (RFC 8707), refresh and revocation (RFC 7009).

use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;
use url::form_urlencoded::Serializer;

use crate::config::secure_or_loopback;
use crate::error::{Error, Result};
use crate::http;

/// The client name shown on the consent page.
const CLIENT_NAME: &str = "generative-icons";
/// Scope requested when the server advertises none.
const DEFAULT_SCOPE: &str = "mcp:full";

/// The authorization server's endpoints for one engine, as discovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthServer {
    /// Issuer identifier.
    pub issuer: String,
    /// Where the browser is sent to sign in.
    pub authorization_endpoint: String,
    /// Code exchange and refresh.
    pub token_endpoint: String,
    /// Dynamic client registration, if offered.
    pub registration_endpoint: Option<String>,
    /// Token revocation, if offered.
    pub revocation_endpoint: Option<String>,
    /// The protected resource (the engine URL) named in every authorization and token request.
    pub resource: String,
    /// Space-separated scopes to request.
    pub scope: String,
}

/// A dynamically registered public client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredClient {
    /// The `client_id` the server issued.
    pub client_id: String,
    /// Unix seconds; registrations expire after 30 days.
    pub registered_at: u64,
    /// The issuer that registered it.
    pub issuer: String,
}

impl AuthServer {
    /// Checks that these endpoints may receive credentials for `engine`: the issuer and every
    /// endpoint use https (http only on loopback), the endpoints share the issuer's origin, and
    /// the protected resource is `engine` (or its origin).
    pub fn check(&self, engine: &Url) -> Result<()> {
        let issuer = trusted_url("issuer", &self.issuer)?;
        let endpoints = [
            ("authorization endpoint", Some(&self.authorization_endpoint)),
            ("token endpoint", Some(&self.token_endpoint)),
            ("registration endpoint", self.registration_endpoint.as_ref()),
            ("revocation endpoint", self.revocation_endpoint.as_ref()),
        ];
        for (what, endpoint) in endpoints {
            let Some(endpoint) = endpoint else { continue };
            if trusted_url(what, endpoint)?.origin() != issuer.origin() {
                return Err(Error::Protocol(format!(
                    "the {what} {endpoint} is not on the issuer's origin {}",
                    issuer.origin().ascii_serialization()
                )));
            }
        }
        let resource = Url::parse(&self.resource)
            .map_err(|e| Error::Protocol(format!("protected resource {:?}: {e}", self.resource)))?;
        if !same_resource(&resource, engine) {
            return Err(Error::Protocol(format!(
                "the protected resource is {}, not the engine {engine}",
                self.resource
            )));
        }
        Ok(())
    }
}

/// `raw` parsed, when credentials may go there (https, or http on loopback).
fn trusted_url(what: &str, raw: &str) -> Result<Url> {
    let url = Url::parse(raw).map_err(|e| Error::Protocol(format!("{what} {raw:?}: {e}")))?;
    if !secure_or_loopback(&url) {
        return Err(Error::Protocol(format!(
            "the {what} {raw} must use https (http is allowed for loopback only)"
        )));
    }
    Ok(url)
}

/// Whether `resource` names `engine`: the same URL (ignoring a trailing `/`), or its origin
/// when the metadata describes the whole host.
fn same_resource(resource: &Url, engine: &Url) -> bool {
    let trimmed = |url: &Url| url.as_str().trim_end_matches('/').to_owned();
    resource.origin() == engine.origin()
        && (trimmed(resource) == trimmed(engine)
            || (resource.path() == "/" && resource.query().is_none()))
}

/// A successful token response (RFC 6749 §5.1).
#[derive(Clone, Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_in: Option<u64>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redacted = |token: &Option<String>| token.as_ref().map(|_| "[redacted]");
        f.debug_struct("TokenResponse")
            .field("access_token", &"[redacted]")
            .field("expires_in", &self.expires_in)
            .field("refresh_token", &redacted(&self.refresh_token))
            .field("refresh_token_expires_in", &self.refresh_token_expires_in)
            .field("scope", &self.scope)
            .field("id_token", &redacted(&self.id_token))
            .finish()
    }
}

#[derive(Deserialize)]
struct ProtectedResource {
    resource: Option<String>,
    #[serde(default)]
    authorization_servers: Vec<String>,
    #[serde(default)]
    scopes_supported: Vec<String>,
}

#[derive(Deserialize)]
struct ServerMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    registration_endpoint: Option<String>,
    revocation_endpoint: Option<String>,
    code_challenge_methods_supported: Option<Vec<String>>,
    #[serde(default)]
    scopes_supported: Vec<String>,
}

/// Unix seconds now.
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Finds the authorization server for `engine`: protected-resource metadata first (path form,
/// then root form), then the issuer's authorization-server metadata (RFC 8414, then OIDC).
/// The result passes [`AuthServer::check`], and the server must support PKCE with S256.
pub(crate) fn discover(client: &Client, engine: &Url) -> Result<AuthServer> {
    let resource_meta: ProtectedResource = first_json(client, &protected_resource_urls(engine))?;
    let issuer = resource_meta
        .authorization_servers
        .first()
        .cloned()
        .ok_or_else(|| {
            Error::Protocol("protected-resource metadata lists no authorization server".into())
        })?;
    let resource = resource_meta
        .resource
        .ok_or_else(|| Error::Protocol("protected-resource metadata names no resource".into()))?;
    // Checked before anything is fetched from the issuer.
    let issuer_url = trusted_url("issuer", &issuer)?;
    let meta: ServerMetadata = first_json(client, &authorization_server_urls(&issuer_url))?;
    if meta.issuer.trim_end_matches('/') != issuer.trim_end_matches('/') {
        return Err(Error::Protocol(format!(
            "authorization server metadata names issuer {:?}, expected {issuer:?}",
            meta.issuer
        )));
    }
    // The MCP authorization spec: refuse servers that do not advertise S256.
    let methods = meta.code_challenge_methods_supported.unwrap_or_default();
    if !methods.iter().any(|m| m == "S256") {
        return Err(Error::Protocol(
            "the authorization server does not advertise PKCE S256".into(),
        ));
    }
    let scopes = if resource_meta.scopes_supported.is_empty() {
        meta.scopes_supported
    } else {
        resource_meta.scopes_supported
    };
    let server = AuthServer {
        issuer: meta.issuer,
        authorization_endpoint: meta.authorization_endpoint,
        token_endpoint: meta.token_endpoint,
        registration_endpoint: meta.registration_endpoint,
        revocation_endpoint: meta.revocation_endpoint,
        resource,
        scope: if scopes.is_empty() {
            DEFAULT_SCOPE.to_owned()
        } else {
            scopes.join(" ")
        },
    };
    server.check(engine)?;
    Ok(server)
}

fn protected_resource_urls(engine: &Url) -> Vec<Url> {
    well_known(engine, "oauth-protected-resource")
}

fn authorization_server_urls(issuer: &Url) -> Vec<Url> {
    let mut urls = well_known(issuer, "oauth-authorization-server");
    urls.extend(well_known(issuer, "openid-configuration"));
    if !issuer.path().trim_end_matches('/').is_empty() {
        // OIDC Discovery appends to the issuer path.
        let mut appended = issuer.clone();
        appended.set_path(&format!(
            "{}/.well-known/openid-configuration",
            issuer.path().trim_end_matches('/')
        ));
        urls.push(appended);
    }
    urls
}

/// `https://host/.well-known/<name><path>`, then `https://host/.well-known/<name>` for a path.
fn well_known(base: &Url, name: &str) -> Vec<Url> {
    let path = base.path().trim_end_matches('/');
    let mut root = base.clone();
    root.set_query(None);
    root.set_fragment(None);
    let mut urls = Vec::new();
    if !path.is_empty() {
        let mut with_path = root.clone();
        with_path.set_path(&format!("/.well-known/{name}{path}"));
        urls.push(with_path);
    }
    root.set_path(&format!("/.well-known/{name}"));
    urls.push(root);
    urls
}

/// GETs each URL in turn and parses the first 200 answer; 4xx moves on to the next candidate.
fn first_json<T: for<'de> Deserialize<'de>>(client: &Client, urls: &[Url]) -> Result<T> {
    let mut last = None;
    for url in urls {
        let response = client
            .get(url.clone())
            .header(ACCEPT, "application/json")
            .send()?;
        let status = response.status().as_u16();
        if status == 200 {
            let text = response.text()?;
            return serde_json::from_str(&text).map_err(|e| Error::Protocol(format!("{url}: {e}")));
        }
        if !(400..500).contains(&status) {
            return Err(http::status_error(response));
        }
        last = Some(http::status_error(response));
    }
    Err(last.unwrap_or_else(|| Error::Protocol("no discovery URL".into())))
}

/// Registers a public client for the loopback `redirect_uri`.
pub(crate) fn register(
    client: &Client,
    server: &AuthServer,
    redirect_uri: &str,
) -> Result<RegisteredClient> {
    let endpoint = server.registration_endpoint.as_deref().ok_or_else(|| {
        Error::Protocol(
            "the authorization server does not offer dynamic client registration".into(),
        )
    })?;
    let body = json!({
        "client_name": CLIENT_NAME,
        "redirect_uris": [redirect_uri],
        "grant_types": ["authorization_code", "refresh_token"],
        "response_types": ["code"],
        "token_endpoint_auth_method": "none",
        "scope": server.scope,
    });
    let response = client
        .post(endpoint)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(body.to_string())
        .send()?;
    let value: Value = oauth_json(endpoint, response)?;
    let client_id = value
        .get("client_id")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Protocol("registration response has no client_id".into()))?;
    Ok(RegisteredClient {
        client_id: client_id.to_owned(),
        registered_at: value
            .get("client_id_issued_at")
            .and_then(Value::as_u64)
            .unwrap_or_else(now),
        issuer: server.issuer.clone(),
    })
}

/// A PKCE verifier and its S256 challenge.
pub(crate) struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

impl Pkce {
    pub fn new() -> Result<Self> {
        let verifier = random_token(32)?;
        Ok(Pkce {
            challenge: s256(&verifier),
            verifier,
        })
    }
}

/// `BASE64URL(SHA256(verifier))`.
pub(crate) fn s256(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// `bytes` random bytes, base64url without padding.
pub(crate) fn random_token(bytes: usize) -> Result<String> {
    let mut buf = vec![0u8; bytes];
    getrandom::fill(&mut buf).map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
    Ok(URL_SAFE_NO_PAD.encode(buf))
}

/// The URL the browser opens to sign in.
pub(crate) fn authorization_url(
    server: &AuthServer,
    client_id: &str,
    redirect_uri: &str,
    challenge: &str,
    state: &str,
) -> Result<String> {
    // The browser opens this URL, so never anything but https (or http on loopback).
    let mut url = trusted_url("authorization endpoint", &server.authorization_endpoint)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", &server.scope)
        .append_pair("state", state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("resource", &server.resource);
    Ok(url.into())
}

/// Exchanges an authorization code for tokens.
pub(crate) fn exchange_code(
    client: &Client,
    server: &AuthServer,
    client_id: &str,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse> {
    token_request(
        client,
        server,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("code_verifier", verifier),
            ("resource", &server.resource),
        ],
    )
}

/// Redeems a refresh token. Never retried: the server rotates refresh tokens, and replaying
/// one that it already redeemed revokes the whole chain.
pub(crate) fn refresh(
    client: &Client,
    server: &AuthServer,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse> {
    token_request(
        client,
        server,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("resource", &server.resource),
        ],
    )
}

/// Revokes a refresh token (RFC 7009); the server revokes its access tokens with it.
pub(crate) fn revoke(
    client: &Client,
    endpoint: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<()> {
    let body = form(&[
        ("token", refresh_token),
        ("token_type_hint", "refresh_token"),
        ("client_id", client_id),
    ]);
    let response = client
        .post(endpoint)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(oauth_error(endpoint, response))
    }
}

fn token_request(
    client: &Client,
    server: &AuthServer,
    params: &[(&str, &str)],
) -> Result<TokenResponse> {
    let response = client
        .post(&server.token_endpoint)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(ACCEPT, "application/json")
        .body(form(params))
        .send()?;
    oauth_json(&server.token_endpoint, response)
}

fn form(params: &[(&str, &str)]) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.extend_pairs(params);
    serializer.finish()
}

/// Parses a 2xx JSON body, or turns an error body into [`Error::OAuth`].
fn oauth_json<T: for<'de> Deserialize<'de>>(endpoint: &str, response: Response) -> Result<T> {
    if !response.status().is_success() {
        return Err(oauth_error(endpoint, response));
    }
    let text = response.text()?;
    serde_json::from_str(&text).map_err(|e| Error::Protocol(format!("{endpoint}: {e}")))
}

fn oauth_error(endpoint: &str, response: Response) -> Error {
    let status = response.status().as_u16();
    if http::is_transient(status) {
        return http::status_error(response);
    }
    let url = http::redacted(response.url());
    let text = response.text().unwrap_or_default();
    match serde_json::from_str::<Value>(&text) {
        Ok(body) if body.get("error").and_then(Value::as_str).is_some() => Error::OAuth {
            endpoint: endpoint.to_owned(),
            error: body["error"].as_str().unwrap_or_default().to_owned(),
            description: body
                .get("error_description")
                .and_then(Value::as_str)
                .map(str::to_owned),
        },
        _ => Error::Http {
            status,
            url,
            body: text.chars().take(300).collect(),
        },
    }
}

/// The signed-in account from a JWT access or ID token (`email`, else `sub`), without
/// verifying the signature: it is only shown to the user.
pub(crate) fn account_from_jwt(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let claims: Value =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?)
            .ok()?;
    ["email", "preferred_username", "sub"]
        .iter()
        .find_map(|key| claims.get(*key).and_then(Value::as_str).map(str::to_owned))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s256_matches_rfc7636_appendix_b() {
        assert_eq!(
            s256("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn pkce_verifier_is_43_url_safe_chars() {
        let pkce = Pkce::new().unwrap();
        assert_eq!(pkce.verifier.len(), 43);
        assert!(
            pkce.verifier
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        );
        assert_eq!(pkce.challenge, s256(&pkce.verifier));
    }

    #[test]
    fn well_known_urls() {
        let engine = Url::parse("https://creator.mcp.lottiefiles.com/mcp").unwrap();
        let urls: Vec<String> = protected_resource_urls(&engine)
            .iter()
            .map(Url::to_string)
            .collect();
        assert_eq!(
            urls,
            [
                "https://creator.mcp.lottiefiles.com/.well-known/oauth-protected-resource/mcp",
                "https://creator.mcp.lottiefiles.com/.well-known/oauth-protected-resource",
            ]
        );
        let issuer = Url::parse("https://creator.mcp.lottiefiles.com").unwrap();
        let urls: Vec<String> = authorization_server_urls(&issuer)
            .iter()
            .map(Url::to_string)
            .collect();
        assert_eq!(
            urls,
            [
                "https://creator.mcp.lottiefiles.com/.well-known/oauth-authorization-server",
                "https://creator.mcp.lottiefiles.com/.well-known/openid-configuration",
            ]
        );
        let tenant = Url::parse("https://auth.example.com/tenant1").unwrap();
        let urls: Vec<String> = authorization_server_urls(&tenant)
            .iter()
            .map(Url::to_string)
            .collect();
        assert_eq!(
            urls[0],
            "https://auth.example.com/.well-known/oauth-authorization-server/tenant1"
        );
        assert_eq!(
            urls[4],
            "https://auth.example.com/tenant1/.well-known/openid-configuration"
        );
    }

    #[test]
    fn authorization_url_carries_pkce_state_and_resource() {
        let server = AuthServer {
            issuer: "https://as.example".into(),
            authorization_endpoint: "https://as.example/authorize".into(),
            token_endpoint: "https://as.example/token".into(),
            registration_endpoint: None,
            revocation_endpoint: None,
            resource: "https://engine.example/mcp".into(),
            scope: "mcp:full".into(),
        };
        let url = authorization_url(
            &server,
            "cid",
            "http://127.0.0.1:5000/callback",
            "chal",
            "st",
        )
        .unwrap();
        let parsed = Url::parse(&url).unwrap();
        let pairs: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        assert_eq!(pairs["response_type"], "code");
        assert_eq!(pairs["client_id"], "cid");
        assert_eq!(pairs["redirect_uri"], "http://127.0.0.1:5000/callback");
        assert_eq!(pairs["scope"], "mcp:full");
        assert_eq!(pairs["state"], "st");
        assert_eq!(pairs["code_challenge"], "chal");
        assert_eq!(pairs["code_challenge_method"], "S256");
        assert_eq!(pairs["resource"], "https://engine.example/mcp");
    }

    fn server(base: &str) -> AuthServer {
        AuthServer {
            issuer: base.into(),
            authorization_endpoint: format!("{base}/authorize"),
            token_endpoint: format!("{base}/token"),
            registration_endpoint: Some(format!("{base}/register")),
            revocation_endpoint: Some(format!("{base}/revoke")),
            resource: "https://engine.example/mcp".into(),
            scope: "mcp:full".into(),
        }
    }

    #[test]
    fn endpoints_must_be_secure_on_the_issuers_origin_for_this_engine() {
        let engine = Url::parse("https://engine.example/mcp").unwrap();
        server("https://as.example").check(&engine).unwrap();
        let mut root = server("https://as.example");
        root.resource = "https://engine.example".into();
        root.check(&engine).unwrap();

        let refused = |edit: fn(&mut AuthServer)| {
            let mut server = server("https://as.example");
            edit(&mut server);
            server.check(&engine).unwrap_err()
        };
        let err = refused(|s| {
            s.authorization_endpoint = "file:///System/Applications/Calculator.app".into()
        });
        assert!(err.to_string().contains("must use https"), "{err}");
        let err = refused(|s| s.token_endpoint = "http://as.example/token".into());
        assert!(err.to_string().contains("must use https"), "{err}");
        let err = refused(|s| s.revocation_endpoint = Some("https://evil.example/revoke".into()));
        assert!(err.to_string().contains("issuer's origin"), "{err}");
        let err = refused(|s| s.issuer = "http://as.example".into());
        assert!(err.to_string().contains("issuer"), "{err}");
        let err = refused(|s| s.resource = "https://other.example/mcp".into());
        assert!(err.to_string().contains("protected resource"), "{err}");
        refused(|s| s.resource = "https://engine.example/other".into());

        // Loopback may use plain http (local engines and tests).
        let local = Url::parse("http://127.0.0.1:9/mcp").unwrap();
        let mut loopback = server("http://127.0.0.1:9");
        loopback.resource = "http://127.0.0.1:9/mcp".into();
        loopback.check(&local).unwrap();
    }

    /// An engine whose discovery documents are `resource` and `metadata` (`{base}` is replaced
    /// by the server's address).
    fn discovery(resource: &str, metadata: &str) -> Result<AuthServer> {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mock = rt.block_on(MockServer::start());
        let base = mock.uri();
        let json =
            |text: &str| -> Value { serde_json::from_str(&text.replace("{base}", &base)).unwrap() };
        rt.block_on(
            Mock::given(method("GET"))
                .and(path("/.well-known/oauth-protected-resource/mcp"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json(resource)))
                .mount(&mock),
        );
        rt.block_on(
            Mock::given(method("GET"))
                .and(path("/.well-known/oauth-authorization-server"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json(metadata)))
                .mount(&mock),
        );
        let engine = Url::parse(&format!("{base}/mcp")).unwrap();
        discover(&http::client().unwrap(), &engine)
    }

    const RESOURCE: &str = r#"{"resource": "{base}/mcp", "authorization_servers": ["{base}"]}"#;
    const METADATA: &str = r#"{"issuer": "{base}", "authorization_endpoint": "{base}/authorize",
        "token_endpoint": "{base}/token", "code_challenge_methods_supported": ["S256"]}"#;

    #[test]
    fn discovery_checks_what_the_server_advertises() {
        let server = discovery(RESOURCE, METADATA).unwrap();
        assert!(server.token_endpoint.ends_with("/token"));
        assert_eq!(server.scope, DEFAULT_SCOPE);

        let file_url = METADATA.replace(
            "{base}/authorize",
            "file:///System/Applications/Calculator.app",
        );
        let err = discovery(RESOURCE, &file_url).unwrap_err();
        assert!(err.to_string().contains("authorization endpoint"), "{err}");
        let elsewhere = METADATA.replace("{base}/token", "https://evil.example/token");
        assert!(discovery(RESOURCE, &elsewhere).is_err());
        let no_pkce = METADATA.replace(r#", "code_challenge_methods_supported": ["S256"]"#, "");
        let err = discovery(RESOURCE, &no_pkce).unwrap_err();
        assert!(err.to_string().contains("S256"), "{err}");
        let plain = METADATA.replace(r#"["S256"]"#, r#"["plain"]"#);
        assert!(discovery(RESOURCE, &plain).is_err());
        let no_resource = r#"{"authorization_servers": ["{base}"]}"#;
        let err = discovery(no_resource, METADATA).unwrap_err();
        assert!(err.to_string().contains("no resource"), "{err}");
        let other =
            r#"{"resource": "https://other.example/mcp", "authorization_servers": ["{base}"]}"#;
        assert!(discovery(other, METADATA).is_err());
        let http_issuer =
            r#"{"resource": "{base}/mcp", "authorization_servers": ["http://as.example"]}"#;
        let err = discovery(http_issuer, METADATA).unwrap_err();
        assert!(err.to_string().contains("must use https"), "{err}");
    }

    #[test]
    fn authorization_url_refuses_other_schemes() {
        let mut server = server("https://as.example");
        server.authorization_endpoint = "file:///System/Applications/Calculator.app".into();
        assert!(
            authorization_url(&server, "cid", "http://127.0.0.1:5/callback", "c", "s").is_err()
        );
    }

    #[test]
    fn token_responses_redact_their_tokens() {
        let tokens: TokenResponse = serde_json::from_value(json!({
            "access_token": "at-secret", "refresh_token": "rt-secret", "id_token": "id-secret",
            "expires_in": 300
        }))
        .unwrap();
        let debug = format!("{tokens:?}");
        assert!(
            !debug.contains("secret") && debug.contains("expires_in: Some(300)"),
            "{debug}"
        );
    }

    #[test]
    fn account_from_jwt_payload() {
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"user-1","email":"a@b.c"}"#);
        assert_eq!(
            account_from_jwt(&format!("x.{payload}.y")).as_deref(),
            Some("a@b.c")
        );
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"user-1"}"#);
        assert_eq!(
            account_from_jwt(&format!("x.{payload}.y")).as_deref(),
            Some("user-1")
        );
        assert_eq!(account_from_jwt("opaque-token"), None);
    }
}
