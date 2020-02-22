use super::{
    imgui_component_utils::*, Component, ComponentList, Entity, Name, PrefabMap, SerializationMarker,
    SerializedEntity,
};
use imgui::Ui;

pub trait ComponentBounds {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>);
    fn is_serialized(&self, serialized_entity: &SerializedEntity, active: bool) -> bool;
    fn commit_to_scene(
        &self,
        serialized_entity: &mut SerializedEntity,
        active: bool,
        serialization_marker: &super::ComponentList<super::SerializationMarker>,
    );
    fn uncommit_to_scene(&self, serialized_entity: &mut SerializedEntity);
    fn post_deserialization(&mut self, _: Entity, _: &ComponentList<SerializationMarker>) {}
}

pub trait SerializableComponent: std::fmt::Debug + typename::TypeName + Clone + Default {
    const SERIALIZATION_NAME: once_cell::sync::Lazy<serde_yaml::Value>;
}

pub struct InspectorParameters<'a, 'b> {
    pub ui: &'b imgui::Ui<'a>,
    pub entities: &'b [Entity],
    pub entity_names: &'b ComponentList<Name>,
    pub prefabs: &'b PrefabMap,
    pub uid: &'b str,
    pub is_open: bool,
}

pub trait ComponentListBounds {
    fn expand_list(&mut self);
    fn unset(&mut self, index: &Entity) -> bool;
    fn get_mut(&mut self, index: &Entity) -> Option<&mut dyn ComponentBounds>;
    fn dump_to_log(&self, index: &Entity);
    fn clone_entity(&mut self, index: &Entity, new_entity: &Entity);

    // IMGUI
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>);

    #[must_use]
    fn component_inspector(
        &mut self,
        entity: &Entity,
        current_serialized_entity: Option<&SerializedEntity>,
        current_prefab_parent: Option<&SerializedEntity>,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        prefab_hashmap: &PrefabMap,
        ui: &imgui::Ui<'_>,
        is_open: bool,
    ) -> Option<ComponentInspectorPostAction>;

    #[must_use]
    fn serialization_option(
        &self,
        ui: &Ui<'_>,
        entity_id: &Entity,
        serialized_marker: &ComponentList<super::SerializationMarker>,
    ) -> Option<ComponentInspectorPostAction>;

    fn load_component_into_serialized_entity(
        &self,
        entity: &Entity,
        serialized_entity: &mut super::SerializedEntity,
        serialization_markers: &ComponentList<super::SerializationMarker>,
    );

    fn post_deserialization(&mut self, entity_names: &ComponentList<SerializationMarker>);
}

impl<T> ComponentListBounds for ComponentList<T>
where
    T: ComponentBounds + SerializableComponent + for<'de> serde::Deserialize<'de> + 'static,
{
    fn expand_list(&mut self) {
        self.expand_list();
    }

    fn unset(&mut self, index: &Entity) -> bool {
        self.unset(index)
    }

    fn dump_to_log(&self, index: &Entity) {
        let comp_name = super::imgui_system::typed_text_ui::<T>();
        if let Some(comp) = self.get(index) {
            println!("{}: {:#?}", comp_name, comp);
        } else {
            println!("{}: None", comp_name);
        }
    }

    fn clone_entity(&mut self, original: &Entity, new_entity: &Entity) {
        if self.get(original).is_some() {
            let new_component = self.get(original).unwrap().inner().clone();
            self.set_component(new_entity, new_component);
        }
    }
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>) {
        if imgui::MenuItem::new(&imgui::ImString::new(super::imgui_system::typed_text_ui::<T>()))
            .enabled(self.get(index).is_none())
            .build(ui)
        {
            self.set_component(index, T::default());
        }
    }

    fn component_inspector(
        &mut self,
        entity: &Entity,
        current_serialized_entity: Option<&SerializedEntity>,
        current_prefab_parent: Option<&SerializedEntity>,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        prefab_hashmap: &PrefabMap,
        ui: &Ui<'_>,
        is_open: bool,
    ) -> Option<ComponentInspectorPostAction> {
        if let Some(comp) = self.get_mut(entity) {
            // get our serialization_statuses:
            let serialized_sync_status: SyncStatus = current_serialized_entity
                .map(|se| {
                    if comp.is_serialized(se) {
                        SyncStatus::Synced
                    } else {
                        SyncStatus::OutofSync
                    }
                })
                .unwrap_or_else(|| {
                    if super::scene_system::current_scene_mode() == super::SceneMode::Draft {
                        SyncStatus::Headless
                    } else {
                        SyncStatus::Unsynced
                    }
                });

            let prefab_sync_status: SyncStatus = current_prefab_parent
                .map(|se| {
                    if comp.is_serialized(se) {
                        SyncStatus::Synced
                    } else {
                        SyncStatus::OutofSync
                    }
                })
                .unwrap_or_else(|| {
                    if super::scene_system::current_scene_mode() == super::SceneMode::Draft {
                        SyncStatus::Headless
                    } else {
                        SyncStatus::Unsynced
                    }
                });

            if let Some(action) = super::imgui_system::component_inspector_raw(
                comp,
                serialized_sync_status,
                prefab_sync_status,
                entities,
                entity_names,
                prefab_hashmap,
                ui,
                is_open,
                |inner, ip| inner.entity_inspector(ip),
            ) {
                match action {
                    ComponentInspectorListAction::Delete => {
                        self.unset(entity);
                    }
                    ComponentInspectorListAction::RevertToParentPrefab => {
                        let prefab: T =
                            SerializedEntity::get_serialized_component(current_prefab_parent.unwrap())
                                .unwrap();
                        *comp.inner_mut() = prefab;
                    }
                    ComponentInspectorListAction::ComponentInspectorPostAction(post_action) => {
                        return Some(post_action)
                    }
                }
            }
        }

        None
    }

    fn serialization_option(
        &self,
        ui: &imgui::Ui<'_>,
        entity_id: &Entity,
        serialized_markers: &ComponentList<super::SerializationMarker>,
    ) -> Option<ComponentInspectorPostAction> {
        self.serialization_option_raw(ui, entity_id, serialized_markers)
    }

    fn load_component_into_serialized_entity(
        &self,
        entity: &Entity,
        serialized_entity: &mut super::SerializedEntity,
        serialization_markers: &ComponentList<super::SerializationMarker>,
    ) {
        if let Some(member_component) = self.get(entity) {
            if member_component
                .inner()
                .is_serialized(serialized_entity, member_component.is_active)
                == false
            {
                member_component.inner().commit_to_scene(
                    serialized_entity,
                    member_component.is_active,
                    serialization_markers,
                );
            }
        }
    }

    fn post_deserialization(&mut self, entity_serde: &ComponentList<SerializationMarker>) {
        for this_one in self.iter_mut() {
            let id = this_one.entity_id();
            this_one.inner_mut().post_deserialization(id, entity_serde);
        }
    }

    fn get_mut(&mut self, index: &Entity) -> Option<&mut dyn ComponentBounds> {
        self.get_mut(index).map(|component| component.inner_mut() as _)
    }
}

impl<T> ComponentList<T>
where
    T: ComponentBounds + Default + Clone + typename::TypeName + 'static,
{
    /// Simply a wrapper around creating a new component
    pub fn set_component(&mut self, entity_id: &Entity, new_component: T) {
        self.set(&entity_id, Component::new(&entity_id, new_component));
    }

    /// Simply a wrapper around creating a new component
    pub fn set_component_with_active(&mut self, entity_id: &Entity, new_component: T, active: bool) {
        self.set(
            &entity_id,
            Component::with_active(&entity_id, new_component, active),
        );
    }

    /// Gets a mutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// a mutable reference to that.
    pub fn get_mut_or_default(&mut self, index: &Entity) -> &mut Component<T> {
        if self.get_mut(index).is_none() {
            self.set_component(index, T::default());
        }

        self.get_mut(index).unwrap()
    }

    /// Gets an immutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// an immutable reference to that. This is **slower** than just
    /// `get`, so use that if you can help it.
    pub fn get_or_default(&mut self, index: &Entity) -> &Component<T> {
        if self.get(index).is_none() {
            self.set_component(index, T::default());
        }

        self.get(index).unwrap()
    }
}
