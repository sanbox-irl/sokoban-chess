use super::serialization_util::{self, SCENE_DIRECTORY};
use super::{SerializedEntity, SingletonDatabase};
use failure::Error;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref CURRENT_SCENE: Mutex<String> = Mutex::new("Main".to_string());
    pub static ref NEXT_SCENE: Mutex<Option<String>> = Mutex::new(None);
}

pub fn set_next_scene(name: &str) -> bool {
    if scene_exists(&name) == false {
        return false;
    }

    let mut next_scene_handle = NEXT_SCENE.lock().unwrap();
    *next_scene_handle = Some(name.to_string());

    true
}

pub fn create_scene(scene_name: &str) -> Result<bool, Error> {
    if scene_exists(&scene_name) {
        return Ok(false);
    }

    // Create the Scene Folder
    let scene_path = format!("{}/{}", SCENE_DIRECTORY, scene_name);
    std::fs::create_dir_all(&scene_path)?;

    // Entities Data
    {
        let blank_entity_save_data: Vec<SerializedEntity> = vec![];
        let entity_path = format!(
            "{}/{}",
            scene_path,
            serialization_util::entities::ENTITY_SUBPATH
        );
        serialization_util::save_serialized_file(&blank_entity_save_data, &entity_path)?;
    }

    // Make a blank singleton database!
    {
        let singleton_database_blank: SingletonDatabase = SingletonDatabase::default();
        let singleton_path = format!(
            "{}/{}",
            scene_path,
            serialization_util::singleton_components::SINGLETONS_SUBPATH
        );
        serialization_util::save_serialized_file(&singleton_database_blank, &singleton_path)?;
    }

    Ok(true)
}

pub fn delete_scene(name: &str) -> Result<bool, Error> {
    if scene_exists(&name) == false {
        return Ok(false);
    }

    let path = format!("{}/{}", SCENE_DIRECTORY, name);
    std::fs::remove_dir_all(&path)?;

    Ok(true)
}

fn scene_exists(name: &str) -> bool {
    let path = format!("{}/{}", SCENE_DIRECTORY, name);
    let path = std::path::Path::new(&path);
    path.exists()
}
