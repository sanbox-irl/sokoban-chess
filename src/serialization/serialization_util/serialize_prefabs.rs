use super::*;
const PREFAB_PATH: &str = "assets/serialized_data/prefabs";
use uuid::Uuid;

pub fn path(entity_id: &str) -> String {
    format!("{}/{}.yaml", PREFAB_PATH, entity_id)
}

/// This is a weird function. We essentially are going to pass the prefab
/// through Serde. We occasionally (by making some things Serde ignore),
/// manipulate runtime data to be invisible to Serde, but this can make
/// live prefab instantiation create incorrect data. If you cycle a prefab
/// through this function, the prefab returned will be stripped of all
/// runtime data, as if it was loaded off disk.
pub fn cycle_prefab(prefab: SerializedEntity) -> Result<SerializedEntity, Error> {
    Ok(serde_yaml::from_value(serde_yaml::to_value(prefab)?)?)
}

pub fn serialize_prefab(prefab: &SerializedEntity) -> Result<(), Error> {
    let path = path(&prefab.id.to_string());

    save_serialized_file(&prefab, &path)
}

pub fn load_prefab(prefab_id: &Uuid) -> Result<Option<SerializedEntity>, Error> {
    // ENTITIES
    let prefab: Result<SerializedEntity, _> = load_serialized_file(&path(&prefab_id.to_string()));

    Ok(prefab
        .map_err(|e| error!("Error loading Prefab File: {}", e))
        .map(|ok| {
            assert_eq!(&ok.id, prefab_id);
            ok
        })
        .ok())
}

pub fn load_all_prefabs() -> Result<std::collections::HashMap<uuid::Uuid, SerializedEntity>, Error> {
    let paths = std::fs::read_dir(PREFAB_PATH)?;
    let mut ret = std::collections::HashMap::new();

    for path in paths {
        let path = path?;

        let prefab: SerializedEntity = load_serialized_file(path.path().to_str().unwrap())?;
        ret.insert(prefab.id, prefab);
    }

    Ok(ret)
}
