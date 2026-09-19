use crate::{
    config::prelude::{AppConfig, HookExecType},
    hooks::prelude::{EncryptionHook, HookContext},
    log_info, utils,
};
use aes_gcm::{KeyInit, aead::Aead};
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use inquire::Password;
use std::{
    fs,
    path::{Path, PathBuf},
};

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;
const VALIDATION_MARKER: &[u8] = b"RCLOUD_ENCRYPTED";
const ENCRYPTION_PREFIX: &str = "rcloud-encrypted-";

impl EncryptionHook {
    /// Prompts for the encryption password and verifies it against the stored
    /// Argon2 hash. Returns the validated password so per-file keys can be
    /// derived from it.
    ///
    /// # Errors
    /// Fails if the stored hash is malformed or the password does not match.
    pub fn verify_password(&self) -> anyhow::Result<String> {
        let password_input = Password::new("Enter encryption password:")
            .without_confirmation()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()
            .context("failed to read password input")?;

        let parsed_hash = PasswordHash::new(&self.hash)
            .map_err(|e| anyhow::anyhow!("invalid argon2 hash format in `hash`: {}", e))?;

        Argon2::default()
            .verify_password(password_input.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("invalid password provided"))?;

        Ok(password_input)
    }

    /// Derives a 32-byte AES-256 key from `password` and a per-file `salt`
    /// using Argon2.
    ///
    /// A fresh random salt is used for every encrypted file, so the same
    /// password yields a different key per file and there is no global salt to
    /// precompute against.
    fn derive_key_with_salt(&self, password: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>> {
        let salt = argon2::password_hash::SaltString::encode_b64(salt)
            .map_err(|e| anyhow::anyhow!("failed to encode salt: {}", e))?;

        let output = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("failed to derive key: {}", e))?;

        let hash_output = output
            .hash
            .ok_or_else(|| anyhow::anyhow!("crypto error: missing hash output"))?;

        let key = hash_output.as_bytes().to_vec();

        anyhow::ensure!(key.len() >= 32, "derived key too short");

        Ok(key[..32].to_vec())
    }

    /// Encrypts `data` under a key derived from `password` and a fresh random
    /// per-file salt.
    ///
    /// Output layout: `MARKER (16) | SALT (16) | NONCE (12) | CIPHERTEXT`.
    fn encrypt(&self, data: &[u8], password: &str) -> anyhow::Result<Vec<u8>> {
        let salt_bytes: [u8; SALT_SIZE] = rand::random();
        let derived_key = self.derive_key_with_salt(password, &salt_bytes)?;

        let cipher = aes_gcm::Aes256Gcm::new_from_slice(&derived_key[..32])
            .context("failed to create cipher")?;

        let nonce_bytes: [u8; NONCE_SIZE] = rand::random();
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;

        let mut result = Vec::with_capacity(
            VALIDATION_MARKER.len() + SALT_SIZE + NONCE_SIZE + ciphertext.len(),
        );

        result.extend_from_slice(VALIDATION_MARKER);
        result.extend_from_slice(&salt_bytes);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypts data produced by [`encrypt`](Self::encrypt).
    ///
    /// Reads the per-file salt from the header, derives the matching key from
    /// `password`, and authenticates/decrypts the ciphertext.
    fn decrypt(&self, data: &[u8], password: &str) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(
            data.len() > VALIDATION_MARKER.len() + SALT_SIZE + NONCE_SIZE,
            "encrypted data too short"
        );

        anyhow::ensure!(
            &data[..VALIDATION_MARKER.len()] == VALIDATION_MARKER,
            "data is not encrypted with expected marker"
        );

        let salt_start = VALIDATION_MARKER.len();
        let nonce_start = salt_start + SALT_SIZE;
        let cipher_start = nonce_start + NONCE_SIZE;

        let salt = &data[salt_start..nonce_start];
        let derived_key = self.derive_key_with_salt(password, salt)?;

        let nonce = aes_gcm::Nonce::from_slice(&data[nonce_start..cipher_start]);
        let ciphertext = &data[cipher_start..];

        let cipher = aes_gcm::Aes256Gcm::new_from_slice(&derived_key[..32])
            .context("failed to create cipher")?;

        cipher.decrypt(nonce, ciphertext).map_err(|e| {
            anyhow::anyhow!("decryption failed: wrong password or corrupt data: {}", e)
        })
    }

    fn process_file(
        &self,
        source: &PathBuf,
        password: &str,
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        let data =
            std::fs::read(source).with_context(|| format!("failed to read file: {:?}", source))?;

        let processed = match self.exec {
            HookExecType::Push => self.encrypt(&data, password)?,
            HookExecType::Pull => self.decrypt(&data, password)?,
        };

        let file_name = source.file_name().context("failed to get file name")?;
        let file_name = file_name.to_string_lossy();

        let new_name = match self.exec {
            HookExecType::Push => format!("{}.enc", file_name),
            HookExecType::Pull => {
                if let Some(stripped) = file_name.strip_suffix(".enc") {
                    stripped.to_string()
                } else {
                    file_name.into_owned()
                }
            }
        };

        let output_path = output_dir.join(new_name);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create parent directory {} for {}",
                    parent.display(),
                    source.display()
                )
            })?;
        }

        fs::write(&output_path, processed)
            .with_context(|| format!("failed to write processed file: {:?}", output_path))?;

        match self.exec {
            HookExecType::Push => log_info!("encrypted {:?} -> {:?}", source, output_path),
            HookExecType::Pull => log_info!("decrypted {:?} -> {:?}", source, output_path),
        }

        Ok(())
    }

    fn process_directory(
        &self,
        source: &PathBuf,
        password: &str,
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        for entry in walkdir::WalkDir::new(source)
            .into_iter()
            .filter_map(Result::ok)
        {
            let entry_path = entry.path();
            if entry_path == source {
                continue;
            }

            let relative_path = entry_path
                .strip_prefix(source)
                .context("failed to build relative path")?;

            let output_path = output_dir.join(relative_path);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&output_path).with_context(|| {
                    format!("failed to create directory: {}", output_path.display())
                })?;

                continue;
            }

            if entry.file_type().is_file() {
                let parent = relative_path.parent().unwrap_or(Path::new(""));
                let target_dir = output_dir.join(parent);
                self.process_file(&entry_path.to_path_buf(), password, &target_dir)?;
            }
        }

        Ok(())
    }

    pub fn process_path(
        &self,
        ctx: &HookContext,
        cfg: &AppConfig,
        password: &str,
    ) -> anyhow::Result<PathBuf> {
        let tempdir = match utils::Directories::tempdir(cfg.core.temp_path.clone())? {
            Some(directory) => tempfile::Builder::new()
                .prefix(ENCRYPTION_PREFIX)
                .tempdir_in(&directory)
                .with_context(|| {
                    format!("failed to create temp directory in {}", directory.display())
                })?,
            None => tempfile::Builder::new()
                .prefix(ENCRYPTION_PREFIX)
                .tempdir()
                .context("failed to create temp directory")?,
        };

        if ctx.path.is_dir() {
            let dir_name = ctx
                .path
                .file_name()
                .context("failed to resolve directory name")?
                .to_string_lossy();

            let normalized_dir_name = match self.exec {
                HookExecType::Push => dir_name.into_owned(),
                HookExecType::Pull => dir_name
                    .strip_suffix(".enc")
                    .map(str::to_string)
                    .unwrap_or_else(|| dir_name.into_owned()),
            };

            let output_root = tempdir.path().join(&normalized_dir_name);
            fs::create_dir_all(&output_root).with_context(|| {
                format!(
                    "failed to create root directory {} in temp output",
                    output_root.display()
                )
            })?;

            self.process_directory(&ctx.path, password, &output_root)?;

            return Ok(tempdir.keep().join(normalized_dir_name));
        }

        self.process_file(&ctx.path, password, tempdir.path())?;
        let kept_dir = tempdir.keep();

        let entries: Vec<_> = std::fs::read_dir(&kept_dir)?
            .map(|e| e.unwrap().path())
            .collect();

        if entries.len() == 1 {
            return Ok(entries[0].clone());
        }

        Ok(kept_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::prelude::HookExecType;

    fn hook() -> EncryptionHook {
        // `hash` is only consulted by `verify_password`; encrypt/decrypt derive
        // per-file keys straight from the password, so a placeholder is fine.
        EncryptionHook {
            exec: HookExecType::Push,
            hash: String::new(),
        }
    }

    #[test]
    fn encrypt_decrypt_round_trip() -> anyhow::Result<()> {
        let h = hook();
        let plaintext = b"the quick brown fox";
        let password = "correct horse battery staple";

        let encrypted = h.encrypt(plaintext, password)?;
        let decrypted = h.decrypt(&encrypted, password)?;

        assert_eq!(decrypted, plaintext);
        Ok(())
    }

    #[test]
    fn header_layout_and_marker() -> anyhow::Result<()> {
        let encrypted = hook().encrypt(b"data", "pw")?;

        assert_eq!(&encrypted[..VALIDATION_MARKER.len()], VALIDATION_MARKER);
        // marker + salt + nonce + non-empty ciphertext (AES-GCM adds a tag).
        assert!(encrypted.len() > VALIDATION_MARKER.len() + SALT_SIZE + NONCE_SIZE);
        Ok(())
    }

    #[test]
    fn random_salt_per_file() -> anyhow::Result<()> {
        let h = hook();
        let a = h.encrypt(b"same", "pw")?;
        let b = h.encrypt(b"same", "pw")?;

        let salt_a = &a[VALIDATION_MARKER.len()..VALIDATION_MARKER.len() + SALT_SIZE];
        let salt_b = &b[VALIDATION_MARKER.len()..VALIDATION_MARKER.len() + SALT_SIZE];

        // Same plaintext + password must not produce identical output: the
        // per-file random salt (and nonce) differ.
        assert_ne!(salt_a, salt_b);
        assert_ne!(a, b);
        Ok(())
    }

    #[test]
    fn wrong_password_fails() -> anyhow::Result<()> {
        let h = hook();
        let encrypted = h.encrypt(b"secret", "right-password")?;

        assert!(h.decrypt(&encrypted, "wrong-password").is_err());
        Ok(())
    }

    #[test]
    fn rejects_short_or_unmarked_data() {
        let h = hook();
        assert!(h.decrypt(b"too short", "pw").is_err());

        let mut bogus = vec![0u8; VALIDATION_MARKER.len() + SALT_SIZE + NONCE_SIZE + 8];
        bogus[..VALIDATION_MARKER.len()].copy_from_slice(b"WRONG_MARKER_XX!");
        assert!(h.decrypt(&bogus, "pw").is_err());
    }
}
