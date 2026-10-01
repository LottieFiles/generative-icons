//! Credential storage.
//!
//! `<home>/credentials.json` (mode 0600, in a 0700 home) holds, per engine origin, the
//! discovered endpoints, the registered client and the current session. The long-lived refresh
//! token goes to the OS keychain (service `gen-animated-icons`, account = engine origin plus a
//! hash of the home folder, so each home has its own) when one is available; otherwise it stays
//! in the file. The access token lives five minutes and stays in the file.
//!
//! Earlier builds kept the refresh token under the bare engine origin, one entry for every home.
//! A home whose file predates per-home accounts (version 1) owns it: its first locked load moves
//! the token to this home's account, and [`CredentialStore::take_legacy`] removes one that could
//! not be moved, so it can be revoked before the file is rewritten. Other homes never touch it.
//!
//! A record is used only for the engine URL it was signed in to, and only while its endpoints
//! pass [`AuthServer::check`] for that URL, so no token goes to another origin.
//!
//! The keychain is read and written only under `<home>/refresh.lock`, an exclusive file lock
//! shared by all processes, so a rotated refresh token is saved before anyone else can read the
//! old one.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::{CredentialStoreKind, EngineConfig};
use crate::error::{Error, Result};
use crate::oauth::{AuthServer, RegisteredClient, TokenResponse, account_from_jwt, now};

#[cfg_attr(test, allow(dead_code))]
const KEYCHAIN_SERVICE: &str = "gen-animated-icons";
const CREDENTIALS_FILE: &str = "credentials.json";
const LOCK_FILE: &str = "refresh.lock";
/// The credentials file format. Version 2 files were written by builds that keep the refresh
/// token under the per-home keychain account; older ones may have it under the bare origin.
const FILE_VERSION: u32 = 2;
/// Lifetime assumed when a token response omits `expires_in` (the engine issues 5-minute tokens).
const DEFAULT_ACCESS_LIFETIME: u64 = 300;

/// Everything stored for one engine.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EngineRecord {
    pub engine: String,
    pub server: AuthServer,
    pub client: Option<RegisteredClient>,
    pub session: Option<Session>,
}

/// A signed-in session.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Session {
    pub access_token: String,
    /// Unix seconds.
    pub access_token_expires_at: u64,
    /// Present in memory; in the file only when no keychain is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Unix seconds, when the server says.
    pub refresh_token_expires_at: Option<u64>,
    pub scope: Option<String>,
    pub account: Option<String>,
}

impl Session {
    /// The session after a token response; a missing refresh token keeps the previous one.
    pub fn from_tokens(tokens: TokenResponse, previous: Option<&Session>) -> Self {
        let issued = now();
        let rotated = tokens.refresh_token.is_some();
        let refresh_token = tokens
            .refresh_token
            .or_else(|| previous.and_then(|p| p.refresh_token.clone()));
        let refresh_token_expires_at = match tokens.refresh_token_expires_in {
            Some(secs) => Some(issued + secs),
            None if !rotated => previous.and_then(|p| p.refresh_token_expires_at),
            None => None,
        };
        let account = tokens
            .id_token
            .as_deref()
            .and_then(account_from_jwt)
            .or_else(|| account_from_jwt(&tokens.access_token))
            .or_else(|| previous.and_then(|p| p.account.clone()));
        Session {
            access_token_expires_at: issued + tokens.expires_in.unwrap_or(DEFAULT_ACCESS_LIFETIME),
            access_token: tokens.access_token,
            refresh_token,
            refresh_token_expires_at,
            scope: tokens.scope,
            account,
        }
    }

    /// Whether the access token stays valid for at least `secs` more seconds.
    pub fn valid_for(&self, secs: u64) -> bool {
        self.access_token_expires_at > now() + secs
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("access_token_expires_at", &self.access_token_expires_at)
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[redacted]"),
            )
            .field("account", &self.account)
            .finish_non_exhaustive()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct CredentialsFile {
    version: u32,
    engines: BTreeMap<String, EngineRecord>,
}

/// Secrets by account name, under the service `gen-animated-icons`.
pub(crate) trait Keychain: Send + Sync {
    /// The secret stored for `account`, if any.
    fn get(&self, account: &str) -> Result<Option<String>>;
    /// Stores `secret` for `account`, replacing what was there.
    fn set(&self, account: &str, secret: &str) -> Result<()>;
    /// Deletes the secret for `account`; deleting a missing one is fine.
    fn delete(&self, account: &str) -> Result<()>;
}

/// The OS keychain (macOS Keychain, Windows Credential Manager, Secret Service). Unit tests
/// use a fake instead.
#[cfg_attr(test, allow(dead_code))]
struct OsKeychain;

#[cfg_attr(test, allow(dead_code))]
impl OsKeychain {
    fn entry(account: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(KEYCHAIN_SERVICE, account).map_err(keychain_error)
    }
}

impl Keychain for OsKeychain {
    fn get(&self, account: &str) -> Result<Option<String>> {
        match Self::entry(account)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(keychain_error(e)),
        }
    }

    fn set(&self, account: &str, secret: &str) -> Result<()> {
        Self::entry(account)?
            .set_password(secret)
            .map_err(keychain_error)
    }

    fn delete(&self, account: &str) -> Result<()> {
        match Self::entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(keychain_error(e)),
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
fn keychain_error(error: keyring::Error) -> Error {
    Error::Store(format!("keychain: {error}"))
}

/// The keychain an `Auto` store uses: the OS one when it is available.
#[cfg(not(test))]
fn auto_keychain(_config: &EngineConfig) -> Option<Arc<dyn Keychain>> {
    keyring::Entry::store_status()
        .is_ok()
        .then(|| Arc::new(OsKeychain) as Arc<dyn Keychain>)
}

/// Unit tests never touch the OS keychain: an `Auto` store uses the config's fake, if any.
#[cfg(test)]
fn auto_keychain(config: &EngineConfig) -> Option<Arc<dyn Keychain>> {
    config.test_keychain.clone()
}

/// The credentials of one engine.
pub(crate) struct CredentialStore {
    home: PathBuf,
    /// The engine URL; a stored record is used only when it was signed in to this one.
    engine: Url,
    origin: String,
    keychain: Option<Arc<dyn Keychain>>,
}

impl CredentialStore {
    pub fn open(config: &EngineConfig) -> Self {
        let keychain = match config.credential_store {
            CredentialStoreKind::File => None,
            CredentialStoreKind::Auto => auto_keychain(config),
        };
        CredentialStore {
            home: config.home.clone(),
            engine: config.url.clone(),
            origin: config.origin(),
            keychain,
        }
    }

    /// `"keychain"` or `"file"`: where the refresh token lives.
    pub fn backend(&self) -> &'static str {
        if self.keychain.is_some() {
            "keychain"
        } else {
            "file"
        }
    }

    /// The record from the file alone, without the lock or the keychain: enough for the session's
    /// state and the registered client. With a keychain, its refresh token is `None`.
    pub fn record(&self) -> Result<Option<EngineRecord>> {
        Ok(self.record_in(self.read_file()?))
    }

    /// The record with its refresh token; `None` when there is none, or when the stored one
    /// belongs to another URL on this origin or names endpoints this engine may not send tokens
    /// to. A refresh token that an earlier build left under the legacy keychain account moves to
    /// this home's account, unless a build of this format has written the file since.
    pub fn load(&self, _lock: &RefreshLock) -> Result<Option<EngineRecord>> {
        let file = self.read_file()?;
        let legacy_era = file.version < FILE_VERSION;
        let Some(mut record) = self.record_in(file) else {
            return Ok(None);
        };
        if let (Some(keychain), Some(session)) = (&self.keychain, record.session.as_mut()) {
            let account = self.keychain_account();
            session.refresh_token = match keychain.get(&account)? {
                Some(token) => Some(token),
                None if legacy_era => self.move_legacy(keychain.as_ref(), &account)?,
                None => None,
            };
        }
        Ok(Some(record))
    }

    fn record_in(&self, mut file: CredentialsFile) -> Option<EngineRecord> {
        file.engines
            .remove(&self.origin)
            .filter(|record| self.issued_for_this_engine(record))
    }

    /// Moves the legacy entry's token to `account`. The legacy entry is deleted only once the
    /// token is safe under the new name; if that delete fails, `login` and `logout` remove it.
    fn move_legacy(&self, keychain: &dyn Keychain, account: &str) -> Result<Option<String>> {
        let Some(token) = keychain.get(self.legacy_account())? else {
            return Ok(None);
        };
        keychain.set(account, &token)?;
        let _ = keychain.delete(self.legacy_account());
        Ok(Some(token))
    }

    /// Deletes the refresh token an earlier build left under the legacy keychain account and
    /// returns it, so the caller can revoke it. That entry was shared by every home, so only a
    /// home whose file predates per-home accounts and holds a record for this engine claims it;
    /// for any other home (a new one, or one this build has written) it stays untouched.
    pub fn take_legacy(&self, _lock: &RefreshLock) -> Result<Option<String>> {
        let Some(keychain) = &self.keychain else {
            return Ok(None);
        };
        let file = self.read_file()?;
        if file.version >= FILE_VERSION || self.record_in(file).is_none() {
            return Ok(None);
        }
        let token = keychain.get(self.legacy_account())?;
        if token.is_some() {
            keychain.delete(self.legacy_account())?;
        }
        Ok(token)
    }

    fn issued_for_this_engine(&self, record: &EngineRecord) -> bool {
        record.engine == self.engine.as_str()
            && record.server.check(&self.engine).is_ok()
            && record
                .client
                .as_ref()
                .is_none_or(|client| client.issuer == record.server.issuer)
    }

    /// Writes `record`.
    pub fn save(&self, _lock: &RefreshLock, record: &EngineRecord) -> Result<()> {
        ensure_dir(&self.home)?;
        let mut stored = record.clone();
        if let Some(keychain) = &self.keychain {
            let account = self.keychain_account();
            match stored.session.as_mut().and_then(|s| s.refresh_token.take()) {
                Some(token) => keychain.set(&account, &token)?,
                None => keychain.delete(&account)?,
            }
        }
        let mut file = self.read_file()?;
        file.version = FILE_VERSION;
        file.engines.insert(self.origin.clone(), stored);
        self.write_file(&file)
    }

    /// Takes the cross-process lock that guards refreshes and writes.
    pub fn lock(&self) -> Result<RefreshLock> {
        ensure_dir(&self.home)?;
        let file = private_options()
            .read(true)
            .write(true)
            .truncate(false)
            .open(self.home.join(LOCK_FILE))?;
        fs4::FileExt::lock(&file)?;
        Ok(RefreshLock(file))
    }

    /// The engine origin and a hash of the canonical home, so homes never share a token.
    fn keychain_account(&self) -> String {
        let home = fs::canonicalize(&self.home).unwrap_or_else(|_| self.home.clone());
        let hash = gai_core::sha256_hex(home.as_os_str().as_encoded_bytes());
        format!("{} {}", self.origin, &hash[..16])
    }

    /// The account earlier builds used for every home: the bare engine origin.
    fn legacy_account(&self) -> &str {
        &self.origin
    }

    fn path(&self) -> PathBuf {
        self.home.join(CREDENTIALS_FILE)
    }

    fn read_file(&self) -> Result<CredentialsFile> {
        match fs::read(self.path()) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| Error::Store(format!("{}: {e}", self.path().display()))),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(CredentialsFile::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Replaces the file atomically: a new temp file with a random name (created exclusively,
    /// mode 0600, never through a symlink), then a rename.
    fn write_file(&self, file: &CredentialsFile) -> Result<()> {
        ensure_dir(&self.home)?;
        let text = serde_json::to_string_pretty(file).map_err(|e| Error::Store(e.to_string()))?;
        let mut out = tempfile::Builder::new()
            .prefix(&format!(".{CREDENTIALS_FILE}."))
            .suffix(".tmp")
            .tempfile_in(&self.home)?;
        out.write_all(text.as_bytes())?;
        out.write_all(b"\n")?;
        out.as_file().sync_all()?;
        out.persist(self.path()).map_err(|e| e.error)?;
        Ok(())
    }
}

/// Held while refreshing or writing credentials; released on drop.
pub(crate) struct RefreshLock(File);

impl Drop for RefreshLock {
    fn drop(&mut self) {
        // Closing the file releases the lock as well.
        let _ = fs4::FileExt::unlock(&self.0);
    }
}

fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.create(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options
}

/// Creates `dir` (mode 0700), or makes an existing one private to its owner.
fn ensure_dir(dir: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(dir)?.permissions().mode() & 0o077 != 0 {
            // Fails unless this user owns the folder.
            fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
                Error::Store(format!(
                    "{} is open to other users and cannot be made private: {e}",
                    dir.display()
                ))
            })?;
        }
    }
    Ok(())
}

/// An in-memory keychain for unit tests.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct FakeKeychain {
    pub entries: std::sync::Mutex<BTreeMap<String, String>>,
    /// Makes every `set` fail, like a locked or denied keychain.
    pub refuse_writes: std::sync::atomic::AtomicBool,
}

#[cfg(test)]
impl FakeKeychain {
    pub fn entries(&self) -> BTreeMap<String, String> {
        self.entries.lock().unwrap().clone()
    }
}

#[cfg(test)]
impl Keychain for FakeKeychain {
    fn get(&self, account: &str) -> Result<Option<String>> {
        Ok(self.entries.lock().unwrap().get(account).cloned())
    }

    fn set(&self, account: &str, secret: &str) -> Result<()> {
        if self
            .refuse_writes
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(Error::Store("keychain: write refused".into()));
        }
        self.entries
            .lock()
            .unwrap()
            .insert(account.to_owned(), secret.to_owned());
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<()> {
        self.entries.lock().unwrap().remove(account);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A signed-in record for the engine at `https://<host>/mcp`.
    fn record(host: &str, refresh: Option<&str>) -> EngineRecord {
        EngineRecord {
            engine: format!("https://{host}/mcp"),
            server: AuthServer {
                issuer: format!("https://{host}"),
                authorization_endpoint: format!("https://{host}/authorize"),
                token_endpoint: format!("https://{host}/token"),
                registration_endpoint: None,
                revocation_endpoint: None,
                resource: format!("https://{host}/mcp"),
                scope: "mcp:full".into(),
            },
            client: Some(RegisteredClient {
                client_id: "cid".into(),
                registered_at: 1,
                issuer: format!("https://{host}"),
            }),
            session: Some(Session {
                access_token: "at".into(),
                access_token_expires_at: now() + 300,
                refresh_token: refresh.map(str::to_owned),
                refresh_token_expires_at: None,
                scope: None,
                account: Some("a@b.c".into()),
            }),
        }
    }

    fn file_store(home: &Path, url: &str) -> CredentialStore {
        let mut config = EngineConfig::new(url, home).unwrap();
        config.credential_store = CredentialStoreKind::File;
        CredentialStore::open(&config)
    }

    /// A store for `https://engine.example/mcp` whose refresh tokens go to `keychain`.
    fn keychain_store(home: &Path, keychain: &Arc<FakeKeychain>) -> CredentialStore {
        let mut config = EngineConfig::new("https://engine.example/mcp", home).unwrap();
        config.test_keychain = Some(keychain.clone());
        CredentialStore::open(&config)
    }

    fn load(store: &CredentialStore) -> Option<EngineRecord> {
        store.load(&store.lock().unwrap()).unwrap()
    }

    fn save(store: &CredentialStore, record: &EngineRecord) {
        store.save(&store.lock().unwrap(), record).unwrap();
    }

    fn refresh_token(store: &CredentialStore) -> Option<String> {
        load(store)?.session?.refresh_token
    }

    /// What a build before per-home accounts left behind: a version 1 file and the refresh token
    /// under the bare origin.
    fn legacy_sign_in(home: &Path, keychain: &FakeKeychain, token: &str) {
        let stored = record("engine.example", None);
        let file = serde_json::json!({
            "version": 1,
            "engines": {"https://engine.example": serde_json::to_value(&stored).unwrap()},
        });
        fs::create_dir_all(home).unwrap();
        fs::write(home.join(CREDENTIALS_FILE), file.to_string()).unwrap();
        keychain
            .entries
            .lock()
            .unwrap()
            .insert("https://engine.example".into(), token.into());
    }

    #[test]
    fn file_store_round_trips_per_origin_with_private_modes() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let store = file_store(&home, "https://engine.example/mcp");
        let other = file_store(&home, "https://other.example/mcp");
        assert!(load(&store).is_none());
        save(&store, &record("engine.example", Some("rt-1")));
        save(&other, &record("other.example", Some("rt-other")));
        assert_eq!(refresh_token(&store).as_deref(), Some("rt-1"));
        assert_eq!(refresh_token(&other).as_deref(), Some("rt-other"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |p: PathBuf| fs::metadata(p).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode(home.join(CREDENTIALS_FILE)), 0o600);
            assert_eq!(mode(home.join(LOCK_FILE)), 0o600);
            assert_eq!(mode(home.clone()), 0o700);
        }
        let leftovers: Vec<_> = fs::read_dir(&home)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .filter(|name| name.to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
        assert_eq!(store.backend(), "file");
    }

    #[test]
    fn records_are_used_only_for_the_engine_they_were_issued_for() {
        let dir = tempfile::tempdir().unwrap();
        let store = file_store(dir.path(), "https://engine.example/mcp");
        save(&store, &record("engine.example", Some("rt-1")));
        assert!(load(&store).is_some());
        assert!(store.record().unwrap().is_some());

        // Another endpoint on the same origin shares the file key, not the session.
        let sibling = file_store(dir.path(), "https://engine.example/other");
        assert!(load(&sibling).is_none());
        assert!(sibling.record().unwrap().is_none());

        // An edited file that points the token endpoint elsewhere is ignored.
        let mut moved = record("engine.example", Some("rt-1"));
        moved.server.token_endpoint = "https://evil.example/token".into();
        save(&store, &moved);
        assert!(load(&store).is_none());
        assert!(store.record().unwrap().is_none());
        let mut plain = record("engine.example", Some("rt-1"));
        plain.server.revocation_endpoint = Some("http://engine.example/revoke".into());
        save(&store, &plain);
        assert!(load(&store).is_none());
        let mut foreign_client = record("engine.example", Some("rt-1"));
        foreign_client.client.as_mut().unwrap().issuer = "https://evil.example".into();
        save(&store, &foreign_client);
        assert!(load(&store).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn writes_never_follow_a_planted_temp_file_and_tighten_the_home() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o755)).unwrap();
        // The temp name the store used to write through, pointing at another user's file.
        let victim = dir.path().join("victim");
        fs::write(&victim, "untouched").unwrap();
        let planted = home.join(format!(".{CREDENTIALS_FILE}.{}.tmp", std::process::id()));
        std::os::unix::fs::symlink(&victim, &planted).unwrap();

        let store = file_store(&home, "https://engine.example/mcp");
        save(&store, &record("engine.example", Some("rt-1")));
        assert_eq!(fs::read_to_string(&victim).unwrap(), "untouched");
        assert_eq!(refresh_token(&store).as_deref(), Some("rt-1"));
        let mode = fs::metadata(&home).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
    }

    #[test]
    fn each_home_has_its_own_keychain_account() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("a"), dir.path().join("b"));
        fs::create_dir(&a).unwrap();
        fs::create_dir(&b).unwrap();
        let url = "https://engine.example/mcp";
        let account = |home: &Path| file_store(home, url).keychain_account();
        assert_ne!(account(&a), account(&b));
        assert_eq!(account(&a), account(&b.join("../a")));
        assert!(account(&a).starts_with("https://engine.example "));
    }

    #[test]
    fn the_keychain_holds_the_refresh_token_under_this_homes_account() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = Arc::new(FakeKeychain::default());
        let store = keychain_store(dir.path(), &keychain);
        save(&store, &record("engine.example", Some("rt-1")));
        let account = store.keychain_account();
        assert_eq!(
            keychain.entries(),
            BTreeMap::from([(account, "rt-1".into())])
        );
        let file = fs::read_to_string(dir.path().join(CREDENTIALS_FILE)).unwrap();
        assert!(
            !file.contains("rt-1") && file.contains("\"version\": 2"),
            "{file}"
        );
        assert_eq!(refresh_token(&store).as_deref(), Some("rt-1"));
        // The file alone has no refresh token, and reading it needs neither lock nor keychain.
        let record = store.record().unwrap().unwrap();
        assert!(record.session.unwrap().refresh_token.is_none());
        assert_eq!(store.backend(), "keychain");

        let mut signed_out = load(&store).unwrap();
        signed_out.session = None;
        save(&store, &signed_out);
        assert!(keychain.entries().is_empty());
    }

    #[test]
    fn a_legacy_refresh_token_moves_to_this_homes_account() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = Arc::new(FakeKeychain::default());
        legacy_sign_in(dir.path(), &keychain, "rt-legacy");
        let store = keychain_store(dir.path(), &keychain);
        // Reading the file alone touches nothing.
        assert!(store.record().unwrap().unwrap().session.is_some());
        assert_eq!(keychain.entries().len(), 1);

        assert_eq!(refresh_token(&store).as_deref(), Some("rt-legacy"));
        let account = store.keychain_account();
        assert_eq!(
            keychain.entries(),
            BTreeMap::from([(account, "rt-legacy".into())])
        );
        // Moved once: the next load reads the new account.
        assert_eq!(refresh_token(&store).as_deref(), Some("rt-legacy"));
        assert!(store.take_legacy(&store.lock().unwrap()).unwrap().is_none());
    }

    #[test]
    fn a_legacy_refresh_token_stays_put_when_it_cannot_move() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = Arc::new(FakeKeychain::default());
        legacy_sign_in(dir.path(), &keychain, "rt-legacy");
        let store = keychain_store(dir.path(), &keychain);
        keychain
            .refuse_writes
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let err = store.load(&store.lock().unwrap()).err().unwrap();
        assert!(matches!(err, Error::Store(_)), "{err}");
        assert_eq!(
            keychain.entries(),
            BTreeMap::from([("https://engine.example".into(), "rt-legacy".into())])
        );
        keychain
            .refuse_writes
            .store(false, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(refresh_token(&store).as_deref(), Some("rt-legacy"));
    }

    #[test]
    fn only_a_file_from_before_per_home_accounts_claims_the_legacy_token() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = Arc::new(FakeKeychain::default());
        legacy_sign_in(dir.path(), &keychain, "rt-legacy");

        // A home this build signed in to (version 2) whose own entry went missing.
        let other = dir.path().join("other");
        let store = keychain_store(&other, &keychain);
        save(&store, &record("engine.example", Some("rt-other")));
        keychain.delete(&store.keychain_account()).unwrap();
        assert_eq!(refresh_token(&store), None);

        // A home without a session never claims it either.
        let empty = keychain_store(&dir.path().join("empty"), &keychain);
        assert!(load(&empty).is_none());

        // When this home's account already has a token, the legacy one is left for take_legacy.
        let current = keychain_store(dir.path(), &keychain);
        keychain
            .set(&current.keychain_account(), "rt-current")
            .unwrap();
        assert_eq!(refresh_token(&current).as_deref(), Some("rt-current"));
        let lock = current.lock().unwrap();
        assert_eq!(
            current.take_legacy(&lock).unwrap().as_deref(),
            Some("rt-legacy")
        );
        assert!(current.take_legacy(&lock).unwrap().is_none());
        assert_eq!(keychain.entries().len(), 1);
    }

    #[test]
    fn session_keeps_refresh_token_when_not_rotated() {
        let previous = record("engine.example", Some("rt-1")).session.unwrap();
        let tokens = TokenResponse {
            access_token: "at-2".into(),
            expires_in: Some(300),
            refresh_token: None,
            refresh_token_expires_in: None,
            scope: Some("mcp:full".into()),
            id_token: None,
        };
        let next = Session::from_tokens(tokens, Some(&previous));
        assert_eq!(next.refresh_token.as_deref(), Some("rt-1"));
        assert_eq!(next.account.as_deref(), Some("a@b.c"));
        assert!(next.valid_for(200) && !next.valid_for(400));
        assert!(!format!("{next:?}").contains("rt-1"));
    }
}
