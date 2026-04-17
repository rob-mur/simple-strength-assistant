use serde::{Deserialize, Serialize};

/// Sync credentials read from OPFS/LocalStorage.
/// These are written by the pairing flow (#90) or auto-generated on first
/// launch (#148) and read here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncCredentials {
    /// UUID identifying the sync slot on the server
    pub sync_id: String,
    /// Secret used to authenticate requests (sent as X-Sync-Secret header — never in URL)
    pub sync_secret: String,
    /// UUID identifying this specific device
    pub device_id: String,
}

/// Key used to store/retrieve sync credentials in LocalStorage.
const CREDS_KEY: &str = "sync_credentials";

/// Key used to store/retrieve the vector clock in LocalStorage.
const CLOCK_KEY: &str = "sync_vector_clock";

impl SyncCredentials {
    /// Load credentials from LocalStorage, returning `None` if not present.
    pub fn load() -> Option<Self> {
        #[cfg(not(test))]
        {
            use gloo_storage::{LocalStorage, Storage};
            LocalStorage::get::<SyncCredentials>(CREDS_KEY).ok()
        }
        #[cfg(test)]
        {
            // In unit tests there is no LocalStorage; tests inject credentials directly.
            let _ = CREDS_KEY;
            None
        }
    }

    /// Load credentials from LocalStorage, generating and persisting new ones
    /// if none exist yet.  This is called on first app launch to bootstrap
    /// sync without user interaction (#148).
    #[cfg(not(test))]
    pub fn load_or_generate() -> Self {
        if let Some(existing) = Self::load() {
            return existing;
        }
        let creds = Self::generate();
        if let Err(e) = creds.save() {
            log::warn!("[Sync] Failed to persist auto-generated credentials: {}", e);
        }
        creds
    }

    /// Generate fresh sync credentials using random UUIDs.
    /// `sync_id` and `device_id` are UUID-v4; `sync_secret` is set to a
    /// placeholder value because the backend currently uses sync_id-as-credential
    /// auth (no secret required).
    pub fn generate() -> Self {
        let sync_id = uuid::Uuid::new_v4().to_string();
        let device_id = uuid::Uuid::new_v4().to_string();
        // The sync_secret field is required by is_valid() but the backend does
        // not enforce it yet.  Use a generated UUID so the credential passes
        // validation and is ready when the backend adds secret-based auth.
        let sync_secret = uuid::Uuid::new_v4().to_string();
        Self {
            sync_id,
            sync_secret,
            device_id,
        }
    }

    /// Persist credentials to LocalStorage.
    #[cfg(not(test))]
    pub fn save(&self) -> Result<(), String> {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::set(CREDS_KEY, self).map_err(|e| e.to_string())
    }

    /// Validates that none of the credential fields are empty and that
    /// `sync_id` contains only URL-safe characters (alphanumeric + hyphen).
    /// This is stricter than blocking individual characters and prevents
    /// path-traversal, query injection, and fragment injection.
    pub fn is_valid(&self) -> bool {
        !self.sync_id.is_empty()
            && !self.sync_secret.is_empty()
            && !self.device_id.is_empty()
            && self
                .sync_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
    }
}

// ── Vector clock persistence ─────────────────────────────────────────────

use super::VectorClock;

/// Persist a vector clock to LocalStorage so it survives page reloads.
pub fn save_clock(clock: &VectorClock) -> Result<(), String> {
    #[cfg(not(test))]
    {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::set(CLOCK_KEY, clock).map_err(|e| e.to_string())
    }
    #[cfg(test)]
    {
        let _ = (clock, CLOCK_KEY);
        Ok(())
    }
}

/// Load a vector clock from LocalStorage, returning a fresh clock if not present.
pub fn load_clock() -> VectorClock {
    #[cfg(not(test))]
    {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::get::<VectorClock>(CLOCK_KEY).unwrap_or_default()
    }
    #[cfg(test)]
    {
        let _ = CLOCK_KEY;
        VectorClock::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_credentials() {
        let creds = SyncCredentials {
            sync_id: "abc".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(creds.is_valid());
    }

    #[test]
    fn test_invalid_credentials_empty_sync_id() {
        let creds = SyncCredentials {
            sync_id: "".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_invalid_credentials_empty_secret() {
        let creds = SyncCredentials {
            sync_id: "abc".into(),
            sync_secret: "".into(),
            device_id: "device-1".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_invalid_credentials_empty_device_id() {
        let creds = SyncCredentials {
            sync_id: "abc".into(),
            sync_secret: "secret".into(),
            device_id: "".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_load_returns_none_in_test_environment() {
        // In test mode there is no browser LocalStorage; load must return None
        assert!(SyncCredentials::load().is_none());
    }

    #[test]
    fn test_invalid_credentials_sync_id_with_slash() {
        let creds = SyncCredentials {
            sync_id: "../admin".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_invalid_credentials_sync_id_with_dot() {
        let creds = SyncCredentials {
            sync_id: "..".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_invalid_credentials_sync_id_with_query() {
        let creds = SyncCredentials {
            sync_id: "abc?admin=true".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_invalid_credentials_sync_id_with_hash() {
        let creds = SyncCredentials {
            sync_id: "abc#fragment".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_invalid_credentials_sync_id_with_ampersand() {
        let creds = SyncCredentials {
            sync_id: "abc&x=1".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn test_valid_credentials_uuid_format() {
        let creds = SyncCredentials {
            sync_id: "550e8400-e29b-41d4-a716-446655440000".into(),
            sync_secret: "secret".into(),
            device_id: "device-1".into(),
        };
        assert!(creds.is_valid());
    }

    #[test]
    fn test_generate_produces_valid_credentials() {
        let creds = SyncCredentials::generate();
        assert!(creds.is_valid(), "Generated credentials must be valid");
        assert!(!creds.sync_id.is_empty());
        assert!(!creds.sync_secret.is_empty());
        assert!(!creds.device_id.is_empty());
    }

    #[test]
    fn test_generate_produces_unique_ids() {
        let a = SyncCredentials::generate();
        let b = SyncCredentials::generate();
        assert_ne!(
            a.sync_id, b.sync_id,
            "Each call must produce a unique sync_id"
        );
        assert_ne!(
            a.device_id, b.device_id,
            "Each call must produce a unique device_id"
        );
    }

    #[test]
    fn test_generated_sync_id_is_uuid_format() {
        let creds = SyncCredentials::generate();
        // UUID v4 format: 8-4-4-4-12 hex chars separated by hyphens
        assert_eq!(creds.sync_id.len(), 36);
        assert!(
            creds
                .sync_id
                .chars()
                .all(|c| c.is_ascii_hexdigit() || c == '-')
        );
    }

    #[test]
    fn test_load_clock_returns_default_in_test() {
        let clock = super::load_clock();
        assert_eq!(clock, super::VectorClock::new());
    }

    #[test]
    fn test_save_clock_succeeds_in_test() {
        let clock = super::VectorClock::new();
        assert!(super::save_clock(&clock).is_ok());
    }
}
