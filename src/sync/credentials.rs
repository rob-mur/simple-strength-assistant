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

    /// Validates that none of the credential fields are empty.
    pub fn is_valid(&self) -> bool {
        !self.sync_id.is_empty() && !self.sync_secret.is_empty() && !self.device_id.is_empty()
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
}
