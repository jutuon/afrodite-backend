use std::{
    collections::HashMap,
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use error_stack::{Report, Result, ResultExt};
use serde::{Deserialize, Serialize};
use simple_backend_utils::ContextExt;
use url::Url;

use crate::GetConfigError;

pub type GoogleAccountId = String;

pub const CONFIG_FILE_NAME: &str = "simple_backend_config.toml";

// Optional configs not in default file for safety:
// debug = false
//

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

[socket]
public_api = "127.0.0.1:3000"
internal_api = "127.0.0.1:3001"

[data]
dir = "data"

[[data.sqlite]]
name = "current"

[[data.sqlite]]
name = "history"

# [manager]
# address = "http://127.0.0.1:5000"
# api_key = "TODO"

# [tile_map]
# tile_dir = "/map_tiles"

# [sign_in_with_google]
# client_id_android = "id"
# client_id_ios = "id"
# client_id_server = "id"
# admin_google_account_id = "TODO"

# [tls]
# public_api_cert = "server_config/public_api.cert"
# public_api_key = "server_config/public_api.key"
# internal_api_cert = "server_config/internal_api.cert"
# internal_api_key = "server_config/internal_api.key"
# root_certificate = "server_config/root_certificate"

# [media_backup]
# ssh_address = "user@192.168.64.1"
# target_location = "/home/user/media_backup"
# ssh_private_key = "/home/local/.ssh/id_ed25519"
# rsync_time = "7:00"

# Backup SQLite database files using litestream tool
# [litestream]
# binary = "/usr/bin/litestream"
# config_file = "litestream.yml"

"#;

#[derive(thiserror::Error, Debug)]
pub enum ConfigFileError {
    #[error("Save config file failed")]
    Save,
    #[error("Save default")]
    SaveDefault,
    #[error("Not a directory")]
    NotDirectory,
    #[error("Load config file")]
    LoadConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimpleBackendConfigFile {
    pub debug: Option<bool>,
    pub data: DataConfig,
    pub socket: SocketConfig,
    pub tile_map: Option<TileMapConfig>,
    pub manager: Option<AppManagerConfig>,
    pub sign_in_with_google: Option<SignInWithGoogleConfig>,
    /// TLS is required if debug setting is false.
    pub tls: Option<TlsConfig>,

    pub media_backup: Option<MediaBackupConfig>,
    pub litestream: Option<LitestreamConfig>,
}

impl SimpleBackendConfigFile {
    pub fn load(dir: impl AsRef<Path>) -> Result<SimpleBackendConfigFile, ConfigFileError> {
        let config_string =
            ConfigFileUtils::load_string(dir, CONFIG_FILE_NAME, DEFAULT_CONFIG_FILE_TEXT)?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }
}

pub struct ConfigFileUtils;

impl ConfigFileUtils {
    pub fn save_string(file_path: impl AsRef<Path>, text: &str) -> Result<(), ConfigFileError> {
        let mut file = std::fs::File::create(file_path).change_context(ConfigFileError::Save)?;
        file.write_all(text.as_bytes())
            .change_context(ConfigFileError::Save)?;
        Ok(())
    }

    pub fn join_dir_path_and_file_name(
        dir: impl AsRef<Path>,
        file_name: &str,
    ) -> Result<PathBuf, ConfigFileError> {
        if !dir.as_ref().is_dir() {
            return Err(Report::new(ConfigFileError::NotDirectory));
        }
        let mut file_path = dir.as_ref().to_path_buf();
        file_path.push(file_name);
        return Ok(file_path);
    }

    pub fn load_string(
        dir: impl AsRef<Path>,
        file_name: &str,
        default: &str,
    ) -> Result<String, ConfigFileError> {
        let file_path = Self::join_dir_path_and_file_name(&dir, file_name)
            .change_context(ConfigFileError::LoadConfig)?;
        if !file_path.exists() {
            Self::save_string(&file_path, default).change_context(ConfigFileError::SaveDefault)?;
        }

        std::fs::read_to_string(&file_path).change_context(ConfigFileError::LoadConfig)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataConfig {
    /// Data directory for SQLite databases and other files.
    pub dir: PathBuf,
    pub sqlite: Vec<SqliteDatabase>,
}

impl DataConfig {
    pub fn get_databases(&self) -> Result<Vec<DatabaseInfo>, GetConfigError> {
        let mut databases = HashMap::<String, DatabaseInfo>::new();
        for db in &self.sqlite {
            let old = databases.insert(db.name.clone(), Into::<DatabaseInfo>::into(db.clone()));
            if old.is_some() {
                return Err(GetConfigError::InvalidConfiguration.report())
                    .attach_printable(format!("Duplicate database name: {}", db.name));
            }
        }

        let databases = databases.values().cloned().collect::<Vec<DatabaseInfo>>();
        Ok(databases)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SqliteDatabase {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum DatabaseInfo {
    Sqlite { name: String },
}

impl DatabaseInfo {
    pub fn file_name(&self) -> String {
        match self {
            Self::Sqlite { name, .. } => name.clone(),
        }
    }

    pub fn to_sqlite_database(&self) -> SqliteDatabase {
        SqliteDatabase {
            name: self.file_name(),
        }
    }
}

impl From<SqliteDatabase> for DatabaseInfo {
    fn from(value: SqliteDatabase) -> Self {
        Self::Sqlite { name: value.name }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SocketConfig {
    pub public_api: SocketAddr,
    pub internal_api: SocketAddr,
}

/// App manager config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppManagerConfig {
    pub address: Url,
    pub api_key: String,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct TileMapConfig {
    /// Directory for map tiles.
    /// Tiles must be stored in z/x/y.png format.
    pub tile_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SignInWithGoogleConfig {
    pub client_id_android: String,
    pub client_id_ios: String,
    pub client_id_server: String,
    pub admin_google_account_id: GoogleAccountId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    pub public_api_cert: PathBuf,
    pub public_api_key: PathBuf,
    pub internal_api_cert: PathBuf,
    pub internal_api_key: PathBuf,
    pub root_certificate: PathBuf,
}

/// Backup media files to remote server using SSH
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaBackupConfig {
    /// For example "user@host"
    pub ssh_address: SshAddress,
    /// Target media backup location on remote server.
    pub target_location: PathBuf,
    pub ssh_private_key: AbsolutePathNoWhitespace,
    pub rsync_time: TimeValue,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "String")]
pub struct SshAddress {
    pub username: String,
    pub address: String,
}

impl TryFrom<String> for SshAddress {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let values = value.trim().split(&['@']).collect::<Vec<&str>>();
        match values[..] {
            [username, address] => Ok(Self {
                username: username.to_string(),
                address: address.to_string(),
            }),
            _ => Err(format!("Unknown values: {:?}", values)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "String")]
pub struct TimeValue {
    pub hours: u8,
    pub minutes: u8,
}

impl TryFrom<String> for TimeValue {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let iter = value.trim().split(':');
        let values: Vec<&str> = iter.collect();
        match values[..] {
            [hours, minutes] => {
                let hours: u8 = hours
                    .parse()
                    .map_err(|e: std::num::ParseIntError| e.to_string())?;
                let minutes: u8 = minutes
                    .parse()
                    .map_err(|e: std::num::ParseIntError| e.to_string())?;
                Ok(TimeValue { hours, minutes })
            }
            _ => Err(format!("Unknown values: {:?}", values)),
        }
    }
}

/// Config for Litestream SQLite backups
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LitestreamConfig {
    /// Path to Litestream binary.
    pub binary: PathBuf,
    /// Path to Litestream config file.
    pub config_file: PathBuf,
}

/// Absolute path with no whitespace.
/// Also contains only valid UTF-8 characters.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "String")]
pub struct AbsolutePathNoWhitespace {
    pub path: PathBuf,
}

impl TryFrom<String> for AbsolutePathNoWhitespace {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let path = PathBuf::from(value.trim());
        validate_path(&path)?;
        Ok(Self { path })
    }
}

const PATH_CHARACTERS_WHITELIST: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_./";

fn whitelist_chars(input: &str, whitelist: &str) -> String {
    let invalid_chars = input.chars().filter(|&c| !whitelist.contains(c)).collect();
    invalid_chars
}

fn validate_path(input: &Path) -> std::result::Result<(), String> {
    if !input.is_absolute() {
        return Err(format!("Path is not absolute: {}", input.display()));
    }

    let unaccepted = whitelist_chars(
        input.as_os_str().to_string_lossy().as_ref(),
        PATH_CHARACTERS_WHITELIST,
    );
    if !unaccepted.is_empty() {
        return Err(format!(
            "Invalid characters {} in path: {}",
            unaccepted,
            input.display()
        ));
    }

    Ok(())
}