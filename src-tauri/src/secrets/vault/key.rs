use base64::{engine::general_purpose::STANDARD, Engine};
use keyring::{Entry, Error};
use ring::rand::{SecureRandom, SystemRandom};
use zeroize::Zeroizing;

const SERVICE_NAME: &str = "com.zand.knovum";
const VAULT_KEY_NAME: &str = "foundation.vault-key.v1";
const KEY_LENGTH: usize = 32;

pub(super) fn load_or_create() -> Result<Zeroizing<[u8; KEY_LENGTH]>, String> {
    let entry = Entry::new(SERVICE_NAME, VAULT_KEY_NAME).map_err(|error| error.to_string())?;
    match entry.get_password() {
        Ok(encoded) => decode(&encoded),
        Err(Error::NoEntry) => create(&entry),
        Err(error) => Err(error.to_string()),
    }
}

fn create(entry: &Entry) -> Result<Zeroizing<[u8; KEY_LENGTH]>, String> {
    let mut key = Zeroizing::new([0_u8; KEY_LENGTH]);
    SystemRandom::new()
        .fill(key.as_mut())
        .map_err(|_| "could not generate the secret vault key".to_owned())?;
    entry
        .set_password(&STANDARD.encode(&key[..]))
        .map_err(|error| error.to_string())?;
    Ok(key)
}

fn decode(encoded: &str) -> Result<Zeroizing<[u8; KEY_LENGTH]>, String> {
    let bytes = Zeroizing::new(
        STANDARD
            .decode(encoded)
            .map_err(|error| error.to_string())?,
    );
    let key: [u8; KEY_LENGTH] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| "secret vault key has an invalid length".to_owned())?;
    Ok(Zeroizing::new(key))
}
