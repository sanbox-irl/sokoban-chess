const DEV_GAME_CONFIG_PATH: &str = "dev/config.yaml";
use crate::resources::game_config::Config;

pub fn load_config() -> Result<Config, failure::Error> {
    Ok(super::load_serialized_file(DEV_GAME_CONFIG_PATH)?)
}

pub fn serialize_config(s_config: &Config) -> Result<(), failure::Error> {
    super::save_serialized_file(s_config, DEV_GAME_CONFIG_PATH)?;
    Ok(())
}
