use super::{
    serialization_util, Camera, Component, ComponentBounds, ComponentList, Entity, Marker,
    RenderingUtility, ResourcesDatabase, SingletonComponent,
};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SingletonDatabase {
    pub camera: SingletonComponent<Camera>,
    #[serde(skip)]
    pub rendering_utility: RenderingUtility,
    #[serde(skip)]
    pub associated_entities: HashMap<Marker, Entity>,
}

impl SingletonDatabase {
    pub fn new(marker_map: HashMap<Marker, Entity>) -> Result<SingletonDatabase, failure::Error> {
        let mut serialized_singletons: SingletonDatabase =
            serialization_util::singleton_components::load_singleton_database()?;

        serialized_singletons.associated_entities = marker_map;

        info!("✔ Loaded Serialized Singletons");
        Ok(serialized_singletons)
    }

    pub fn save_singleton_markers(&self, entity: &Entity) -> Option<Marker> {
        for (this_marker, this_entity) in &self.associated_entities {
            if this_entity == entity {
                return Some(*this_marker);
            }
        }

        None
    }

    pub fn get_associated_entity<T: ComponentBounds>(
        &self,
        sc: &SingletonComponent<T>,
    ) -> Option<Entity> {
        let marker = sc.marker();

        if let Some(entity) = self.associated_entities.get(&marker) {
            Some(entity.clone())
        } else {
            None
        }
    }

    pub fn find_component_on_list<'a, C: ComponentBounds>(
        &self,
        marker: Marker,
        comp_list: &'a ComponentList<C>,
    ) -> Option<&'a Component<C>> {
        comp_list.get(&self.associated_entities.get(&marker).unwrap())
    }

    pub fn edit_serialized_singleton_database<T: ComponentBounds, F>(
        live_component: &mut SingletonComponent<T>,
        edit_function: F,
    ) -> Result<(), failure::Error>
    where
        F: Fn(&mut SingletonDatabase, &mut SingletonComponent<T>),
    {
        let mut serialized_singletons: SingletonDatabase =
            serialization_util::singleton_components::load_singleton_database()?;

        edit_function(&mut serialized_singletons, live_component);

        serialization_util::singleton_components::serialize_singleton_database(
            &serialized_singletons,
        )
    }

    pub fn initialize_with_runtime_resources(
        &mut self,
        resources: &ResourcesDatabase,
        hwi: &super::HardwareInterface,
    ) {
        info!("Initializing Singletons with Runtime Resources...");

        info!("..Initializing Rendering Utility");
        self.rendering_utility.initialize(resources);

        info!("..Initializing Singletons");
        self.camera.inner_mut().initialize_with_hwi(hwi);

        info!("..Initialized Singletons")
    }
}

impl Default for SingletonDatabase {
    fn default() -> Self {
        SingletonDatabase {
            // @update_singletons
            camera: SingletonComponent::new(Marker::Camera, Camera::default()),
            rendering_utility: RenderingUtility::default(),
            associated_entities: HashMap::new(),
        }
    }
}
