use std::path::{Path, PathBuf};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Deserializer};

use crate::{args::TestMode, file::ConfigFileError};

#[derive(Debug, Default, Deserialize)]
pub struct BotConfigFile {
    pub man_image_dir: Option<PathBuf>,
    pub woman_image_dir: Option<PathBuf>,
    /// Config for user bots
    pub bot_config: BaseBotConfig,
    /// Override config for specific user bots.
    #[serde(default)]
    pub bot: Vec<BotInstanceConfig>,
    pub profile_text_moderation: Option<ProfileTextModerationConfig>,
    pub profile_content_moderation: Option<ProfileContentModerationConfig>,
}

impl BotConfigFile {
    pub fn load_if_bot_mode_or_default(file: impl AsRef<Path>, test_mode: &TestMode) -> Result<BotConfigFile, ConfigFileError> {
        if test_mode.bot_mode().is_none() {
            return Ok(BotConfigFile::default())
        }

        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: BotConfigFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        let validate_common_config = |bot: &BaseBotConfig, id: Option<u16>| {
            let error_location = id.map(|v| format!("Bot ID {} config error.", v)).unwrap_or("Bot config error.".to_string());
            if let Some(age) = bot.age {
                if age < 18 || age > 99 {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "{} Age must be between 18 and 99",
                        error_location
                    ));
                }
            }

            if bot.image.is_some() {
                match bot.img_dir_gender() {
                    Gender::Man => {
                        if config.man_image_dir.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                                .attach_printable(format!("{} Image file name configured but man image directory is not configured", error_location));
                        }
                    }
                    Gender::Woman => {
                        if config.woman_image_dir.is_none() {
                            return Err(ConfigFileError::InvalidConfig)
                                .attach_printable(format!("{} Image file name configured but woman image directory is not configured", error_location));
                        }
                    }
                }
            }

            // TODO: Validate all fields?

            Ok(())
        };

        validate_common_config(&config.bot_config, None)?;

        let mut ids = std::collections::HashSet::<u16>::new();
        for bot in &config.bot {
            validate_common_config(&config.bot_config, Some(bot.id))?;

            if ids.contains(&bot.id) {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("Bot ID {} is defined more than once", bot.id));
            }

            ids.insert(bot.id);
        }

        if let Some(img_dir) = &config.man_image_dir {
            check_imgs_exist(&config, img_dir, Gender::Man)?
        }

        if let Some(img_dir) = &config.woman_image_dir {
            check_imgs_exist(&config, img_dir, Gender::Woman)?
        }

        Ok(config)
    }
}

fn check_imgs_exist(
    config: &BotConfigFile,
    img_dir: &Path,
    gender: Gender,
) -> Result<(), ConfigFileError> {
    let configs = [&config.bot_config].into_iter()
        .chain(config.bot.iter().map(|v| &v.config));

    for bot in configs {
        if bot.img_dir_gender() != gender {
            continue;
        }

        if let Some(img) = &bot.image {
            let img_path = img_dir.join(img);
            if !img_path.is_file() {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable(format!("Image file {:?} does not exist", img_path));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Default, Deserialize)]
pub struct BaseBotConfig {
    pub age: Option<u8>,
    pub gender: Option<Gender>,
    pub name: Option<String>,
    /// Image file name.
    ///
    /// The image is loaded from directory which matches gender config.
    ///
    /// If this is not set and image directory is configured, then random
    /// image from the directory is used as profile image.
    pub image: Option<String>,
    /// Overrides image file configs and use randomly generated single color
    /// image as profile image.
    #[serde(default)]
    pub random_color_image: bool,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
    /// All bots will try to send like to this account ID
    pub send_like_to_account_id: Option<simple_backend_utils::UuidBase64Url>,
    #[serde(default)]
    pub change_visibility: bool,
    #[serde(default)]
    pub change_location: bool,
}

impl BaseBotConfig {
    pub fn get_img(&self, config: &BotConfigFile) -> Option<PathBuf> {
        if let Some(img) = self.image.as_ref() {
            match self.img_dir_gender() {
                Gender::Man => config.man_image_dir.as_ref().map(|dir| dir.join(img)),
                Gender::Woman => config.woman_image_dir.as_ref().map(|dir| dir.join(img)),
            }
        } else {
            None
        }
    }

    pub fn img_dir_gender(&self) -> Gender {
        match self.gender {
            None | Some(Gender::Man) => Gender::Man,
            Some(Gender::Woman) => Gender::Woman,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BotInstanceConfig {
    pub id: u16,
    #[serde(flatten)]
    pub config: BaseBotConfig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gender {
    Man,
    Woman,
}

impl<'de> Deserialize<'de> for Gender {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();

        match s.as_str() {
            "man" => Ok(Gender::Man),
            "woman" => Ok(Gender::Woman),
            _ => Err(serde::de::Error::custom("Invalid value for Gender")),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProfileTextModerationConfig {
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct ProfileContentModerationConfig {
    pub initial_content: bool,
    pub added_content: bool,
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
}
