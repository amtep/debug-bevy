use std::{
    ffi::OsString,
    fs::{File, create_dir_all, read_dir, rename},
    io::{Cursor, Write},
    path::PathBuf,
};

use bevy::prelude::*;
use chrono::{DateTime, NaiveDate, Utc};
use directories::ProjectDirs;
use moonshine_save::{
    load::{LoadWorld, TriggerLoad, load_on_default_event},
    save::{SaveWorld, TriggerSave, save_on_default_event},
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    common::{CultName, CultSymbol},
    constants::{
        AUTOSAVE_INTERVAL,
        files::{PROJECT_DIR_APPLICATION, PROJECT_DIR_ORGANIZATION, PROJECT_DIR_QUALIFIER},
    },
    funds::{Funds, FundsAmount},
    main_menu::{LoadedGame, NewGame},
    state::{GameState, MainSetupSet},
    suspicion::{IntelligenceSuspicion, ScientificSuspicion},
    time::GameDate,
    ui::save_load::warn_no_save,
};

const SEPARATOR: &[u8] = b"\n\nAPOCALYPTOSIS\n";
const EXTENSION: &str = "save";

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (autosave, listen_save_keys).run_if(in_state(GameState::Main)),
    )
    .add_systems(
        OnEnter(GameState::Main),
        first_save
            .run_if(resource_exists::<NewGame>)
            .in_set(MainSetupSet::Save),
    )
    .insert_resource(AutosaveTimer(Timer::new(
        AUTOSAVE_INTERVAL,
        TimerMode::Repeating,
    )))
    .add_observer(save_on_default_event)
    .add_observer(load_on_default_event);
}

#[derive(Serialize, Deserialize)]
pub struct SaveMetadata {
    pub save_timestamp: DateTime<Utc>,
    pub cult_name: String,
    pub cult_symbol: char,
    pub game_date: NaiveDate,
    pub funds: FundsAmount,
}

#[derive(Resource, Deref, Clone, Copy)]
pub struct Campaign(usize);

#[derive(Resource, Deref, DerefMut)]
struct AutosaveTimer(Timer);

#[derive(Error, Debug)]
pub enum SaveLoadError {
    #[error("could not locate user home for project folder")]
    ProjectDirFailed,
    #[error("could not create savegame folder {0}: {1}")]
    CreateDirError(PathBuf, std::io::Error),
    #[error("could not open savegame folder {0}: {1}")]
    ReadDirError(PathBuf, std::io::Error),
    #[error("could not read savegame folder {0}: {1}")]
    ReadEntryError(PathBuf, std::io::Error),
    #[error("could not create save file {0}: {1}")]
    CreateSaveError(PathBuf, std::io::Error),
    #[error("could not write save file {0}: {1}")]
    WriteSaveError(PathBuf, std::io::Error),
    #[error("could not move save file into place {0}: {1}")]
    RenameError(PathBuf, std::io::Error),
    #[error("could not read save file {0}: {1}")]
    ReadSaveError(PathBuf, std::io::Error),
}

fn save_inner(
    mut commands: Commands,
    index: usize,
    metadata: SaveMetadata,
) -> Result<(), SaveLoadError> {
    if let Some(pd) = ProjectDirs::from(
        PROJECT_DIR_QUALIFIER,
        PROJECT_DIR_ORGANIZATION,
        PROJECT_DIR_APPLICATION,
    ) {
        let path = pd
            .data_dir()
            .join(format!("saves/{index}.apocalyptosis.{EXTENSION}"));
        info!("Saving to {}", path.display());
        let temp_path = pd
            .data_dir()
            .join(format!("saves/{index}.apocalyptosis.{EXTENSION}.new"));
        let mut file = File::create(&temp_path)
            .map_err(|e| SaveLoadError::CreateSaveError(temp_path.clone(), e))?;
        file.write_all(
            ron::ser::to_string_pretty(&metadata, PrettyConfig::default())
                .unwrap()
                .as_bytes(),
        )
        .map_err(|e| SaveLoadError::WriteSaveError(temp_path.clone(), e))?;
        file.write_all(SEPARATOR)
            .map_err(|e| SaveLoadError::WriteSaveError(temp_path.clone(), e))?;
        let event = SaveWorld::default_into_stream(file)
            .include_resource::<Funds>()
            .include_resource::<CultName>()
            .include_resource::<CultSymbol>()
            .include_resource::<IntelligenceSuspicion>()
            .include_resource::<ScientificSuspicion>()
            .include_resource::<GameDate>();
        commands.trigger_save(event);
        // TODO: only do this if the save succeeded
        rename(temp_path, &path).map_err(|e| SaveLoadError::RenameError(path.clone(), e))?;
        Ok(())
    } else {
        Err(SaveLoadError::ProjectDirFailed)
    }
}

fn save(
    mut commands: Commands,
    campaign: Option<Res<Campaign>>,
    cult_name: Res<CultName>,
    cult_symbol: Res<CultSymbol>,
    game_date: Res<GameDate>,
    funds: Res<Funds>,
) {
    let index = if let Some(index) = campaign {
        **index
    } else {
        match calc_new_campaign_index() {
            Ok(index) => {
                commands.insert_resource(Campaign(index));
                index
            }
            Err(e) => {
                error!("Save error! could not determine campaign index: {e}");
                commands.spawn(warn_no_save());
                return;
            }
        }
    };
    let metadata = SaveMetadata {
        save_timestamp: Utc::now(),
        cult_name: cult_name.0.clone(),
        cult_symbol: cult_symbol.0,
        game_date: game_date.0,
        funds: funds.0,
    };
    if let Err(e) = save_inner(commands.reborrow(), index, metadata) {
        error!("Save error! {e}");
        commands.spawn(warn_no_save());
    }
}

fn autosave(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut timer: ResMut<AutosaveTimer>,
    campaign: Option<Res<Campaign>>,
    cult_name: Res<CultName>,
    cult_symbol: Res<CultSymbol>,
    game_date: Res<GameDate>,
    funds: Res<Funds>,
) {
    if timer.tick(time.delta()).just_finished() {
        save(
            commands.reborrow(),
            campaign,
            cult_name,
            cult_symbol,
            game_date,
            funds,
        );
    }
}

fn first_save(
    mut commands: Commands,
    campaign: Option<Res<Campaign>>,
    cult_name: Res<CultName>,
    cult_symbol: Res<CultSymbol>,
    game_date: Res<GameDate>,
    funds: Res<Funds>,
) {
    save(
        commands.reborrow(),
        campaign,
        cult_name,
        cult_symbol,
        game_date,
        funds,
    );
}

fn listen_save_keys(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    campaign: Option<Res<Campaign>>,
    cult_name: Res<CultName>,
    cult_symbol: Res<CultSymbol>,
    game_date: Res<GameDate>,
    funds: Res<Funds>,
) {
    if keys.just_pressed(KeyCode::F5) {
        save(
            commands.reborrow(),
            campaign,
            cult_name,
            cult_symbol,
            game_date,
            funds,
        );
    }
}

pub fn load(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    campaign: Campaign,
    content: Vec<u8>,
) {
    // Set the next state early, so that it can be set back to MainMenu if the load fails.
    // It won't take effect till the next frame anyway.
    next_state.set(GameState::Main);
    info!("Loading game {}", *campaign);
    commands.trigger_load(LoadWorld::default_from_stream(Cursor::new(content)));
    commands.insert_resource(campaign);
    commands.insert_resource(LoadedGame);
}

fn list_save_files() -> Result<(PathBuf, Vec<OsString>), SaveLoadError> {
    let mut v = Vec::default();
    if let Some(pd) = ProjectDirs::from(
        PROJECT_DIR_QUALIFIER,
        PROJECT_DIR_ORGANIZATION,
        PROJECT_DIR_APPLICATION,
    ) {
        let save_dir = pd.data_dir().join("saves");
        create_dir_all(&save_dir)
            .map_err(|e| SaveLoadError::CreateDirError(save_dir.to_owned(), e))?;
        for entry in
            read_dir(&save_dir).map_err(|e| SaveLoadError::ReadDirError(save_dir.to_owned(), e))?
        {
            let entry = entry.map_err(|e| SaveLoadError::ReadEntryError(save_dir.to_owned(), e))?;
            if entry.path().extension() != Some(&OsString::from(EXTENSION)) {
                continue;
            }
            v.push(entry.file_name().to_owned());
        }
        Ok((save_dir, v))
    } else {
        Err(SaveLoadError::ProjectDirFailed)
    }
}

/// Examine the savefile filenames to find a new number to save under.
fn calc_new_campaign_index() -> Result<usize, SaveLoadError> {
    let mut max_campaign_index = 0;
    for file_name in list_save_files()?.1 {
        // Parse the leading number in the filename
        if let Some(Ok(index)) = file_name
            .to_string_lossy()
            .split(&['.', '-'])
            .next()
            .map(str::parse)
            && index > max_campaign_index
        {
            max_campaign_index = index;
        }
    }
    Ok(max_campaign_index + 1)
}

pub fn scan_saved_games() -> Result<Vec<(Campaign, SaveMetadata, Vec<u8>)>, SaveLoadError> {
    let mut v = Vec::default();
    let (save_dir, savegames) = list_save_files()?;
    for file_name in savegames {
        // Parse the leading number in the filename
        if let Some(Ok(index)) = file_name
            .to_string_lossy()
            .split(&['.', '-'])
            .next()
            .map(str::parse)
        {
            let path = save_dir.join(file_name);
            let Ok(bytes) = std::fs::read(&path).map_err(|e| {
                let e = SaveLoadError::ReadSaveError(path.clone(), e);
                error!("Skipping save file: {e}");
            }) else {
                continue;
            };
            let Some(p) = bytes
                .windows(SEPARATOR.len())
                .position(|window| window == SEPARATOR)
            else {
                error!("Savefile without metadata: {}", path.display());
                continue;
            };
            let (metadata, content) = (&bytes[..p], &bytes[p + SEPARATOR.len()..]);
            let Ok(metadata) = ron::de::from_bytes(metadata) else {
                error!("Savefile with invalid metadata: {}", path.display());
                continue;
            };
            v.push((Campaign(index), metadata, content.to_owned()));
        }
    }
    Ok(v)
}
