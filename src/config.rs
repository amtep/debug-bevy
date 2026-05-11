use std::fs;
use std::time::Duration;

use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::constants::files::*;
use crate::state::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Load), load_config);
}

const DEFAULT_AUTOSAVE_INTERVAL: Duration = Duration::from_mins(5);

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Config {
    pub auto_save_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_save_interval: DEFAULT_AUTOSAVE_INTERVAL,
        }
    }
}

fn load_config(mut commands: Commands) {
    if let Some(pd) = ProjectDirs::from(
        PROJECT_DIR_QUALIFIER,
        PROJECT_DIR_ORGANIZATION,
        PROJECT_DIR_APPLICATION,
    ) {
        let config_dir_path = pd.config_dir();
        if let Err(e) = fs::create_dir_all(config_dir_path) {
            warn!("cannot create config directory: {e}");
        } else {
            let config_path = pd.config_dir().join("config.toml");
            if config_path.exists() {
                match fs::read(&config_path) {
                    Ok(content) => match toml::from_slice::<Config>(&content) {
                        Ok(config) => {
                            info!("Config loaded");
                            commands.insert_resource(config);
                            return;
                        }
                        Err(e) => {
                            warn!("failed to deserialize config: {e}");
                        }
                    },
                    Err(e) => {
                        warn!("cannot read config file: {e}");
                    }
                }
            }
        }
    }
    info!("Using default config");
    commands.init_resource::<Config>();
}

fn save_config() {
    todo!()
}
