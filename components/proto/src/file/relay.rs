use std::convert::TryInto;
use std::io::{self, Write};
use std::fs::{self, File};
use std::path::Path;

use toml;
use crate::file::pk_string::{public_key_to_string, 
    string_to_public_key, PkStringError};


use crate::net::messages::NetAddressError;
use crate::app_server::messages::RelayAddress;

#[derive(Debug)]
pub enum RelayFileError {
    IoError(io::Error),
    TomlDeError(toml::de::Error),
    TomlSeError(toml::ser::Error),
    PkStringError,
    ParseSocketAddrError,
    InvalidPublicKey,
    NetAddressError(NetAddressError),
}

/// A helper structure for serialize and deserializing RelayAddress.
#[derive(Serialize, Deserialize)]
struct RelayFile {
    public_key: String,
    address: String,
}

impl From<io::Error> for RelayFileError {
    fn from(e: io::Error) -> Self {
        RelayFileError::IoError(e)
    }
}

impl From<toml::de::Error> for RelayFileError {
    fn from(e: toml::de::Error) -> Self {
        RelayFileError::TomlDeError(e)
    }
}

impl From<toml::ser::Error> for RelayFileError {
    fn from(e: toml::ser::Error) -> Self {
        RelayFileError::TomlSeError(e)
    }
}

impl From<PkStringError> for RelayFileError {
    fn from(_e: PkStringError) -> Self {
        RelayFileError::PkStringError
    }
}

impl From<NetAddressError> for RelayFileError {
    fn from(e: NetAddressError) -> Self {
        RelayFileError::NetAddressError(e)
    }
}

/// Load RelayAddress from a file
#[allow(unused)]
pub fn load_relay_from_file(path: &Path) -> Result<RelayAddress, RelayFileError> {
    let data = fs::read_to_string(&path)?;
    let relay_file: RelayFile = toml::from_str(&data)?;

    // Decode public key:
    let public_key = string_to_public_key(&relay_file.public_key)?;

    Ok(RelayAddress {
        public_key,
        address: relay_file.address.try_into()?,
    })
}


/// Store RelayAddress to file
pub fn store_relay_to_file(relay_address: &RelayAddress, path: &Path)
    -> Result<(), RelayFileError> {

    let RelayAddress {ref public_key, ref address} = relay_address;

    let relay_file = RelayFile {
        public_key: public_key_to_string(&public_key),
        address: address.as_str().to_string(),
    };

    let data = toml::to_string(&relay_file)?;

    let mut file = File::create(path)?;
    file.write(&data.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crypto::identity::{PublicKey, PUBLIC_KEY_LEN};

    #[test]
    fn test_relay_file_basic() {
        let relay_file: RelayFile = toml::from_str(r#"
            public_key = 'public_key_string'
            address = 'address_string'
        "#).unwrap();

        assert_eq!(relay_file.public_key, "public_key_string");
        assert_eq!(relay_file.address, "address_string");
    }

    #[test]
    fn test_store_load_relay() {
        // Create a temporary directory:
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("relay_address_file");

        let relay_address = RelayAddress {
            public_key: PublicKey::from(&[0xaa; PUBLIC_KEY_LEN]),
            address: "127.0.0.1:1337".to_owned().try_into().unwrap(),
        };

        store_relay_to_file(&relay_address, &file_path).unwrap();
        let relay_address2 = load_relay_from_file(&file_path).unwrap();

        assert_eq!(relay_address, relay_address2);
    }
}


