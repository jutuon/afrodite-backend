//! Mount secure file storage if needed
//!

use std::{
    process::{ExitStatus, Stdio},
    sync::Arc,
};

use error_stack::{Result, ResultExt};
use manager_model::SecureStorageEncryptionKey;
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::{error, info, warn};

use super::{app::S, state::MountStateStorage};
use crate::{
    api::GetApiManager, utils::ContextExt
};

use manager_config::{file::SecureStorageConfig, Config};

#[derive(thiserror::Error, Debug)]
pub enum MountError {
    #[error("Getting key failed")]
    GetKeyFailed,

    #[error("Process start failed")]
    ProcessStartFailed,

    #[error("Process stdin writing failed")]
    ProcessStdinFailed,

    #[error("Command failed with exit status: {0}")]
    CommandFailed(ExitStatus),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MountMode {
    NotMounted,
    MountedWithRemoteKey,
    MountedWithLocalKey,
    MountedWithDefaultKey,
    /// Secure storage was mounted before manager mode
    /// started, so key is unknown.
    MountedWithUnknownKey,
}

#[derive(Debug, Clone)]
pub struct MountState {
    mode: MountMode,
}

impl MountState {
    pub fn new() -> Self {
        Self {
            mode: MountMode::NotMounted,
        }
    }

    pub fn mode(&self) -> MountMode {
        self.mode
    }

    fn set_mode(&mut self, mode: MountMode) {
        self.mode = mode;
    }
}

impl Default for MountState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MountManager {
    config: Arc<Config>,
    state: S,
    mount_state: Arc<MountStateStorage>,
}

impl MountManager {
    pub fn new(config: Arc<Config>, state: S, mount_state: Arc<MountStateStorage>) -> Self {
        Self {
            config,
            state,
            mount_state,
        }
    }

    pub async fn mount_if_needed(
        &self,
        storage_config: &SecureStorageConfig,
    ) -> Result<(), MountError> {
        if storage_config.availability_check_path.exists() {
            info!("Secure storage is already mounted");
            self.mount_state
                .modify(|s| s.mount_state.set_mode(MountMode::MountedWithUnknownKey))
                .await;
            return Ok(());
        }

        let key = self.state.api_manager()
            .get_encryption_key()
            .await
            .change_context(MountError::GetKeyFailed);

        let (key, mut mode) = match key {
            Ok(key) => (Some(key), MountMode::MountedWithRemoteKey),
            Err(e) => {
                error!("Getting encryption key failed: {:?}", e);
                if let Some(text) = &storage_config.encryption_key_text {
                    warn!("Using local encryption key. This shouldn't be done in production!");
                    (
                        Some(SecureStorageEncryptionKey {
                            key: text.to_string(),
                        }),
                        MountMode::MountedWithLocalKey,
                    )
                } else {
                    (None, MountMode::NotMounted)
                }
            }
        };

        match key {
            Some(key) => {
                if self.is_default_password().await? {
                    info!("Default password is used. Password will be changed.");
                    self.change_default_password(key.clone()).await?;
                }
                self.mount_secure_storage(key).await?;
            }
            None => {
                if self.is_default_password().await? {
                    warn!("Mounting secure storage using default password");
                    self.mount_secure_storage(SecureStorageEncryptionKey {
                        key: "password\n".to_string(),
                    })
                    .await?;
                    mode = MountMode::MountedWithDefaultKey;
                } else {
                    return Err(MountError::GetKeyFailed.report());
                }
            }
        };

        self.mount_state.modify(|s| s.mount_state.set_mode(mode)).await;

        Ok(())
    }

    pub async fn mount_secure_storage(&self, key: SecureStorageEncryptionKey) -> Result<(), MountError> {
        let script = self.config.script_locations().open_encryption();

        if !script.exists() {
            warn!("Script for mounting secure storage does not exist");
            return Ok(());
        }

        let mut c = Command::new("sudo")
            .arg(script)
            .stdin(Stdio::piped())
            .spawn()
            .change_context(MountError::ProcessStartFailed)?;

        if let Some(stdin) = c.stdin.as_mut() {
            stdin
                .write_all(key.key.as_bytes())
                .await
                .change_context(MountError::ProcessStdinFailed)?;
            stdin
                .shutdown()
                .await
                .change_context(MountError::ProcessStdinFailed)?;
        }

        let status = c
            .wait()
            .await
            .change_context(MountError::ProcessStartFailed)?;

        if status.success() {
            info!("Mounting was successfull.");
            Ok(())
        } else {
            error!("Mounting failed.");
            Err(MountError::CommandFailed(status).report())
        }
    }

    pub async fn unmount_if_needed(
        &self,
        storage_config: &SecureStorageConfig,
    ) -> Result<(), MountError> {
        if !storage_config.availability_check_path.exists() {
            info!("Secure storage is already unmounted");
            return Ok(());
        }

        info!("Unmounting secure storage");

        let script = self.config.script_locations().close_encryption();

        if !script.exists() {
            warn!("Script for unmounting secure storage does not exist");
            return Ok(());
        }

        // Run command.
        let c = Command::new("sudo")
            .arg(script)
            .status()
            .await
            .change_context(MountError::ProcessStartFailed)?;

        if c.success() {
            info!("Unmounting was successfull.");
            Ok(())
        } else {
            error!("Unmounting failed.");
            Err(MountError::CommandFailed(c).report())
        }
    }

    async fn is_default_password(&self) -> Result<bool, MountError> {
        let script = self.config
            .script_locations()
            .is_default_encryption_password();

        if !script.exists() {
            warn!("Script for checking secure storage password does not exist");
            return Ok(true);
        }

        let c = Command::new("sudo")
            .arg(script)
            .status()
            .await
            .change_context(MountError::ProcessStartFailed)?;

        Ok(c.success())
    }

    async fn change_default_password(&self, key: SecureStorageEncryptionKey) -> Result<(), MountError> {
        let script = self.config.script_locations().change_encryption_password();

        if !script.exists() {
            warn!("Script for changing secure storage password does not exist");
            return Ok(());
        }

        let mut c = Command::new("sudo")
            .arg(script)
            .stdin(Stdio::piped())
            .spawn()
            .change_context(MountError::ProcessStartFailed)?;

        if let Some(stdin) = c.stdin.as_mut() {
            stdin
                .write_all(key.key.as_bytes())
                .await
                .change_context(MountError::ProcessStdinFailed)?;
            stdin
                .shutdown()
                .await
                .change_context(MountError::ProcessStdinFailed)?;
        }

        let status = c
            .wait()
            .await
            .change_context(MountError::ProcessStartFailed)?;

        if status.success() {
            info!("Password change was successfull.");
            Ok(())
        } else {
            error!("Password change failed.");
            Err(MountError::CommandFailed(status).report())
        }
    }
}
