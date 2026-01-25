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
const VALIDATION_MARKER: &[u8] = b"RCLOUD_ENCRYPTED";
const ENCRYPTION_PREFIX: &str = "rcloud-encrypted-";
const STATIC_SALT: &str = "6f2i3mHvfcugVNuT3di+yN+Cc0/v1j6AY2mh04m+Azox";

impl EncryptionHook {
    pub fn derive_key(&self) -> anyhow::Result<Vec<u8>> {
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

        let salt = argon2::password_hash::SaltString::from_b64(STATIC_SALT)
            .map_err(|e| anyhow::anyhow!("invalid static salt {}", e))?;

        let output = Argon2::default()
            .hash_password(password_input.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("failed to derive key: {}", e))?;

        let hash_output = output
            .hash
            .ok_or_else(|| anyhow::anyhow!("crypto error: missing hash output"))?;

        let key = hash_output.as_bytes().to_vec();

        anyhow::ensure!(key.len() >= 32, "derived key too short");

        Ok(key[..32].to_vec())
    }

    fn encrypt(&self, data: &[u8], derived_key: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = aes_gcm::Aes256Gcm::new_from_slice(&derived_key[..32])
            .context("failed to create cipher")?;

        let nonce_bytes: [u8; NONCE_SIZE] = rand::random();
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;

        let mut result =
            Vec::with_capacity(VALIDATION_MARKER.len() + NONCE_SIZE + ciphertext.len());

        result.extend_from_slice(VALIDATION_MARKER);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    fn decrypt(&self, data: &[u8], derived_key: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(
            data.len() > VALIDATION_MARKER.len() + NONCE_SIZE,
            "encrypted data too short"
        );

        anyhow::ensure!(
            &data[..VALIDATION_MARKER.len()] == VALIDATION_MARKER,
            "data is not encrypted with expected marker"
        );

        let nonce_start = VALIDATION_MARKER.len();
        let cipher_start = nonce_start + NONCE_SIZE;

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
        derived_key: &[u8],
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        let data =
            std::fs::read(source).with_context(|| format!("failed to read file: {:?}", source))?;

        let processed = match self.exec {
            HookExecType::Push => self.encrypt(&data, derived_key)?,
            HookExecType::Pull => self.decrypt(&data, derived_key)?,
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
        derived_key: &[u8],
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
                self.process_file(&entry_path.to_path_buf(), derived_key, &target_dir)?;
            }
        }

        Ok(())
    }

    pub fn process_path(
        &self,
        ctx: &HookContext,
        cfg: &AppConfig,
        derived_key: &[u8],
    ) -> anyhow::Result<PathBuf> {
        let tempdir = match utils::Directories::tempdir(cfg.core.temp_path.clone())? {
            Some(directory) => tempfile::Builder::new()
                .prefix(ENCRYPTION_PREFIX)
                .tempdir_in(&directory)
                .with_context(|| format!("failed to create temp directory in {}", &directory.display()))?,
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

            self.process_directory(&ctx.path, derived_key, &output_root)?;

            return Ok(tempdir.keep().join(normalized_dir_name));
        }

        self.process_file(&ctx.path, derived_key, tempdir.path())?;
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
