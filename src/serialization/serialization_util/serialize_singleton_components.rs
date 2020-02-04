use super::*;
pub const SINGLETONS_SUBPATH: &str = "singleton_data.yaml";

pub fn path() -> String {
    format!(
        "{}/{}/{}",
        SCENE_DIRECTORY,
        scene_system::CURRENT_SCENE.lock().unwrap(),
        SINGLETONS_SUBPATH
    )
}

pub fn load_singleton_database() -> Result<SingletonDatabase, Error> {
    load_serialized_file(&path())
}

pub fn serialize_singleton_database(singleton_database: &SingletonDatabase) -> Result<(), Error> {
    save_serialized_file(singleton_database, &path())
}
