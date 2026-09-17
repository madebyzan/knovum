mod key;

use std::{collections::BTreeMap, fs, path::PathBuf};

use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305},
    rand::{SecureRandom, SystemRandom},
};
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"KNOVUMV1";
const NONCE_LENGTH: usize = 12;

pub(super) struct SecretVault {
    path: PathBuf,
    key: Zeroizing<[u8; 32]>,
    records: BTreeMap<String, String>,
}

impl SecretVault {
    pub(super) fn open(path: PathBuf) -> Result<Self, String> {
        let key = key::load_or_create()?;
        let records = if path.exists() {
            decrypt(
                &fs::read(&path).map_err(|error| error.to_string())?,
                key.as_ref(),
            )?
        } else {
            BTreeMap::new()
        };
        Ok(Self { path, key, records })
    }

    pub(super) fn get(&self, key: &str) -> Option<String> {
        self.records.get(key).cloned()
    }

    pub(super) fn set(&mut self, key: String, value: String) -> Result<(), String> {
        let previous = self.records.insert(key.clone(), value);
        if let Err(error) = self.save() {
            restore(&mut self.records, key, previous);
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn delete(&mut self, key: &str) -> Result<(), String> {
        let previous = self.records.remove(key);
        if let Err(error) = self.save() {
            if let Some(value) = previous {
                self.records.insert(key.to_owned(), value);
            }
            return Err(error);
        }
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "secret vault path has no parent".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let bytes = encrypt(&self.records, self.key.as_ref())?;
        let temporary = self.path.with_extension("vault.tmp");
        fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
        replace_file(temporary, &self.path)
    }
}

fn encrypt(records: &BTreeMap<String, String>, key: &[u8]) -> Result<Vec<u8>, String> {
    let mut plaintext = serde_json::to_vec(records).map_err(|error| error.to_string())?;
    let mut nonce = [0_u8; NONCE_LENGTH];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| "could not generate a secret vault nonce".to_owned())?;
    cipher(key)?
        .seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce),
            Aad::from(MAGIC),
            &mut plaintext,
        )
        .map_err(|_| "could not encrypt the secret vault".to_owned())?;
    let mut output = Vec::with_capacity(MAGIC.len() + nonce.len() + plaintext.len());
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&plaintext);
    Ok(output)
}

fn decrypt(bytes: &[u8], key: &[u8]) -> Result<BTreeMap<String, String>, String> {
    if bytes.len() <= MAGIC.len() + NONCE_LENGTH || &bytes[..MAGIC.len()] != MAGIC {
        return Err("secret vault has an invalid format".to_owned());
    }
    let nonce: [u8; NONCE_LENGTH] = bytes[MAGIC.len()..MAGIC.len() + NONCE_LENGTH]
        .try_into()
        .map_err(|_| "secret vault nonce is invalid".to_owned())?;
    let mut ciphertext = bytes[MAGIC.len() + NONCE_LENGTH..].to_vec();
    let plaintext = cipher(key)?
        .open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::from(MAGIC),
            &mut ciphertext,
        )
        .map_err(|_| "secret vault authentication failed".to_owned())?;
    serde_json::from_slice(plaintext).map_err(|error| error.to_string())
}

fn cipher(key: &[u8]) -> Result<LessSafeKey, String> {
    UnboundKey::new(&CHACHA20_POLY1305, key)
        .map(LessSafeKey::new)
        .map_err(|_| "secret vault key is invalid".to_owned())
}

fn restore(records: &mut BTreeMap<String, String>, key: String, previous: Option<String>) {
    if let Some(value) = previous {
        records.insert(key, value);
    } else {
        records.remove(&key);
    }
}

fn replace_file(temporary: PathBuf, target: &PathBuf) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    if target.exists() {
        fs::remove_file(target).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, target).map_err(|error| error.to_string())
}
