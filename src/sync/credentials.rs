use serde::{Deserialize, Serialize};

/// Sync credentials read from OPFS/LocalStorage.
/// These are written by the pairing flow (#90) and read here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncCredentials {
    /// UUID identifying the sync slot on the server
    pub sync_id: String,
    /// Secret used to authenticate requests (HMAC key — never in URL)
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

    /// Persist credentials to LocalStorage.
    #[cfg(not(test))]
    pub fn save(&self) -> Result<(), String> {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::set(CREDS_KEY, self).map_err(|e| e.to_string())
    }

    /// Validates that none of the credential fields are empty and that
    /// `sync_id` does not contain path-traversal characters.
    pub fn is_valid(&self) -> bool {
        !self.sync_id.is_empty()
            && !self.sync_secret.is_empty()
            && !self.device_id.is_empty()
            && !self.sync_id.contains('/')
            && !self.sync_id.contains('.')
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
