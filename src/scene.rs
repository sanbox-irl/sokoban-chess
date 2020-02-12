pub const ENTITY_SUBPATH: &str = "entities_data.yaml";
pub const SCENE_DIRECTORY: &str = "assets/serialized_data/scenes";
pub const PREFAB_DIRECTORY: &str = "assets/serialized_data/prefabs";
pub const SINGLETONS_SUBPATH: &str = "singleton_data.yaml";
pub const DEFAULT_SINGLETONS_SUBPATH: &str = "default_singleton_data.yaml";
pub const TILEMAP_SUBPATH: &str = "tilemap";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    name: String,
    is_prefab: bool,
}

impl Scene {
    pub const fn new(name: String) -> Self {
        Scene {
            name,
            is_prefab: false,
        }
    }

    pub fn new_prefab(prefab_id: uuid::Uuid) -> Self {
        Scene {
            name: prefab_id.to_string(),
            is_prefab: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_prefab(&self) -> bool {
        self.is_prefab
    }

    pub fn entity_path(&self) -> String {
        if self.is_prefab {
            format!("{}/{}.yaml", PREFAB_DIRECTORY, &self.name)
        } else {
            format!("{}/{}/{}", SCENE_DIRECTORY, &self.name, ENTITY_SUBPATH)
        }
    }

    pub fn singleton_path(&self) -> String {
        if self.is_prefab {
            format!("{}/{}", PREFAB_DIRECTORY, DEFAULT_SINGLETONS_SUBPATH)
        } else {
            format!("{}/{}/{}", SCENE_DIRECTORY, &self.name, ENTITY_SUBPATH)
        }
    }

    pub fn tilemap_path(&self, tilemap_path_end: &str) -> String {
        if self.is_prefab {
            format!(
                "{}/{}/{}",
                PREFAB_DIRECTORY, TILEMAP_SUBPATH, tilemap_path_end
            )
        } else {
            format!(
                "{}/{}/{}/{}",
                SCENE_DIRECTORY, &self.name, TILEMAP_SUBPATH, tilemap_path_end
            )
        }
    }
}

use std::fmt;

impl fmt::Display for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            if self.is_prefab { "Prefab" } else { "Scene" },
            self.name
        )
    }
}
