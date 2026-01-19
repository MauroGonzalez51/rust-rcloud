use crate::hooks::prelude::{HookBuilderSharedConfigTrait, HookBuilderTrait};
use anyhow::Context;
use argon2::PasswordHasher;
use inquire::Password;

use crate::{
    config::prelude::{HookConfig, HookExecType, Hooks},
    hooks::encryption::EncryptionHookConfig,
    log_info,
};

impl HookBuilderTrait for EncryptionHookConfig {
    fn build(exec: HookExecType) -> anyhow::Result<HookConfig> {
        log_info!("configuring {} for {}", Hooks::Encryption, exec);

        let password = Self::password()
            .prompt()
            .context("failed to get encryption password")?;

        let hash = Self::derive_hash(&password)?;

        Ok(HookConfig::Encryption(Self { exec, hash }))
    }
}

impl HookBuilderSharedConfigTrait for EncryptionHookConfig {
    fn build_shared() -> anyhow::Result<(HookConfig, HookConfig)> {
        log_info!(
            "configuring {} for {}, {}",
            Hooks::Encryption,
            HookExecType::Push,
            HookExecType::Pull
        );

        let password = Self::password()
            .prompt()
            .context("failed to get encryption password")?;

        let hash = Self::derive_hash(&password)?;

        Ok((
            HookConfig::Encryption(Self {
                exec: HookExecType::Push,
                hash: hash.clone(),
            }),
            HookConfig::Encryption(Self {
                exec: HookExecType::Pull,
                hash: hash.clone(),
            }),
        ))
    }
}

impl EncryptionHookConfig {
    fn derive_hash(password: &str) -> anyhow::Result<String> {
        let salt_bytes: [u8; 16] = rand::random();
        let salt = argon2::password_hash::SaltString::encode_b64(&salt_bytes)
            .map_err(|e| anyhow::anyhow!("failed to create salt: {}", e))?;

        let password_hash = argon2::Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("failed to hash password: {}", e))?;

        Ok(password_hash.to_string())
    }

    fn password() -> Password<'static> {
        Password::new("Enter encryption password:")
            .with_validator(inquire::validator::MinLengthValidator::new(1))
    }
}
