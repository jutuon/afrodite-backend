use std::path::{Path, PathBuf};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Deserializer};
use url::Url;

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
    pub content_moderation: Option<ContentModerationConfig>,
}

impl BotConfigFile {
    pub fn load_if_bot_mode_or_default(
        file: impl AsRef<Path>,
        test_mode: &TestMode,
    ) -> Result<BotConfigFile, ConfigFileError> {
        if test_mode.bot_mode().is_none() {
            return Ok(BotConfigFile::default());
        }

        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let mut config: BotConfigFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        let validate_common_config = |bot: &BaseBotConfig, id: Option<u16>| {
            let error_location = id
                .map(|v| format!("Bot ID {} config error.", v))
                .unwrap_or("Bot config error.".to_string());
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

                if bot.random_color_image.is_some() {
                    return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                        "{} Image and random color image can't be both set",
                        error_location
                    ));
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

        if let Some(config) = &config.profile_text_moderation {
            let count = config
                .user_text_template
                .split(ProfileTextModerationConfig::TEMPLATE_FORMAT_ARGUMENT)
                .count();
            #[allow(clippy::comparison_chain)]
            if count > 2 {
                return Err(ConfigFileError::InvalidConfig)
                    .attach_printable("Profile text moderation user text template: only one '%s' format argument is allowed");
            } else if count < 2 {
                return Err(ConfigFileError::InvalidConfig).attach_printable(
                    "Profile text moderation user text template: '%s' format argument is missing",
                );
            }
        }

        if let Some(config) = &config.content_moderation {
            if let Some(config) = &config.nsfw_detection {
                if !config.model_file.exists() {
                    return Err(ConfigFileError::InvalidConfig)
                        .attach_printable(format!("NSFW model file {} does not exists", config.model_file.display()));
                }
            }
        }

        config.merge_base_bot_config_with_specific_bot_configs();

        Ok(config)
    }

    fn merge_base_bot_config_with_specific_bot_configs(&mut self) {
        for config in &mut self.bot {
            let base = self.bot_config.clone();
            let c = config.config.clone();

            let prevent_base_image_config = c.image.is_some() || c.random_color_image.is_some();
            let base_image = if prevent_base_image_config {
                None
            } else {
                base.image
            };
            let base_random_color_image = if prevent_base_image_config {
                None
            } else {
                base.random_color_image
            };

            config.config = BaseBotConfig {
                age: c.age.or(base.age),
                gender: c.gender.or(base.gender),
                name: c.name.or(base.name),
                image: c.image.or(base_image),
                random_color_image: c.random_color_image.or(base_random_color_image),
                grid_crop_size: c.grid_crop_size.or(base.grid_crop_size),
                grid_crop_x: c.grid_crop_x.or(base.grid_crop_x),
                grid_crop_y: c.grid_crop_y.or(base.grid_crop_y),
                lat: c.lat.or(base.lat),
                lon: c.lon.or(base.lon),
                send_like_to_account_id: c.send_like_to_account_id.or(base.send_like_to_account_id),
                change_visibility: c.change_visibility.or(base.random_color_image),
                change_location: c.change_location.or(base.change_location),
            };
        }
    }
}

fn check_imgs_exist(
    config: &BotConfigFile,
    img_dir: &Path,
    gender: Gender,
) -> Result<(), ConfigFileError> {
    let configs = [&config.bot_config]
        .into_iter()
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

#[derive(Debug, Clone, Default, Deserialize)]
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
    random_color_image: Option<bool>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
    /// Latitude
    pub lat: Option<f64>,
    /// Longitude
    pub lon: Option<f64>,
    /// All bots will try to send like to this account ID
    pub send_like_to_account_id: Option<simple_backend_utils::UuidBase64Url>,
    change_visibility: Option<bool>,
    change_location: Option<bool>,
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

    pub fn random_color_image(&self) -> bool {
        self.random_color_image.unwrap_or_default()
    }

    pub fn change_visibility(&self) -> bool {
        self.change_visibility.unwrap_or_default()
    }

    pub fn change_location(&self) -> bool {
        self.change_location.unwrap_or_default()
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
    /// For example "http://localhost:11434/v1"
    pub openai_api_url: Url,
    pub model: String,
    pub system_text: String,
    /// Format argument "%s" is replaced with profile text.
    pub user_text_template: String,
    /// If LLM response starts with this text or the first
    /// line of the response contains this text, the profile text
    /// is moderated as accepted. The comparisons are case insensitive.
    pub expected_response: String,
    /// Accept all texts which only have single visible character.
    pub accept_single_visible_character: bool,
    pub move_rejected_to_human_moderation: bool,
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
    #[serde(default = "max_tokens_default_value")]
    pub max_tokens: u32,
}

fn max_tokens_default_value() -> u32 {
    10_000
}

impl ProfileTextModerationConfig {
    pub const TEMPLATE_FORMAT_ARGUMENT: &'static str = "%s";
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentModerationDefaultAction {
    Accept,
    Reject,
    MoveToHuman,
}

#[derive(Debug, Deserialize)]
pub struct ContentModerationConfig {
    pub initial_content: bool,
    pub added_content: bool,
    pub moderation_session_max_seconds: u32,
    pub moderation_session_min_seconds: u32,
    /// Skin color based detection.
    /// Actions: reject and move_to_human
    pub nude_detection: Option<NudeDetectionConfig>,
    /// Neural network based detection.
    /// Actions: reject, move_to_human and accept
    pub nsfw_detection: Option<NsfwDetectionConfig>,
    pub default_action: ContentModerationDefaultAction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NudeDetectionConfig {
    pub move_rejected_to_human_moderation: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NsfwDetectionConfig {
    pub model_file: PathBuf,
    /// Thresholds for image rejection.
    pub reject: Option<NsfwDetectionThresholds>,
    /// Thresholds for moving image to human moderation.
    pub move_to_human: Option<NsfwDetectionThresholds>,
    /// Thresholds for accepting the image.
    pub accept: Option<NsfwDetectionThresholds>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NsfwDetectionThresholds {
    pub drawings: Option<f32>,
    pub hentai: Option<f32>,
    pub neutral: Option<f32>,
    pub porn: Option<f32>,
    pub sexy: Option<f32>,
}
