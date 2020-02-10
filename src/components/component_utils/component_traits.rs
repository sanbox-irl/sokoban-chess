use super::{Component, ComponentList, Entity, Name, PrefabMap, SerializedEntity};
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
}

pub struct InspectorParameters<'a, 'b> {
    pub ui: &'b mut imgui::Ui<'a>,
    pub entities: &'b [Entity],
    pub entity_names: &'b ComponentList<Name>,
    pub prefabs: &'b std::collections::HashMap<uuid::Uuid, SerializedEntity>,
    pub uid: &'b str,
    pub is_open: bool,
}

pub enum SyncStatus {
    Synced,
    OutOfSync,
    NA,
}

pub trait ComponentListBounds {
    fn expand_list(&mut self);
    fn unset(&mut self, index: &Entity) -> bool;
    fn dump_to_log(&self, index: &Entity);
    fn clone_entity(&mut self, index: &Entity, new_entity: &Entity);

    // IMGUI
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>);
    fn component_inspector(
        &mut self,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        entity: &Entity,
        prefab_hashmap: &std::collections::HashMap<uuid::Uuid, SerializedEntity>,
        ui: &mut imgui::Ui<'_>,
        is_open: bool,
        // prefab_sync: SyncStatus,
        // serialization_sync: SyncStatus,
    );

    fn serialization_option(
        &self,
        ui: &Ui<'_>,
        entity_id: &Entity,
        is_prefab: bool,
        serialized_marker: &ComponentList<super::SerializationMarker>,
    ) -> failure::Fallible<()>;

    fn create_serialized_entity(
        &self,
        entity: &Entity,
        serialized_entity: &mut super::SerializedEntity,
        serialization_markers: &ComponentList<super::SerializationMarker>,
    );
}

// impl ComponentListBounds for ComponentList<SerializationData> {}
impl<T> ComponentListBounds for ComponentList<T>
where
    T: ComponentBounds + std::fmt::Debug + typename::TypeName + Clone + Default + 'static,
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
        if imgui::MenuItem::new(&imgui::ImString::new(
            super::imgui_system::typed_text_ui::<T>(),
        ))
        .enabled(self.get(index).is_none())
        .build(ui)
        {
            self.set_component(index, T::default());
        }
    }

    fn component_inspector(
        &mut self,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        entity: &Entity,
        prefab_hashmap: &PrefabMap,
        ui: &mut Ui<'_>,
        is_open: bool,
        // prefab_sync: SyncStatus,
        // serialization_sync: SyncStatus,
    ) {
        self.component_inspector_raw(
            entities,
            entity_names,
            entity,
            prefab_hashmap,
            ui,
            is_open,
            |inner, ip| inner.entity_inspector(ip),
        );
    }

    fn serialization_option(
        &self,
        ui: &imgui::Ui<'_>,
        entity_id: &Entity,
        is_prefab: bool,
        serialized_markers: &ComponentList<super::SerializationMarker>,
    ) -> failure::Fallible<()> {
        self.serialization_option_raw(ui, entity_id, is_prefab, serialized_markers)
    }

    fn create_serialized_entity(
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
}

impl<T> ComponentList<T>
where
    T: ComponentBounds + Default + typename::TypeName + 'static,
{
    /// Simply a wrapper around creating a new component
    pub fn set_component(&mut self, entity_id: &Entity, new_component: T) {
        self.set(&entity_id, Component::new(&entity_id, new_component));
    }

    /// Simply a wrapper around creating a new component
    pub fn set_component_with_active(
        &mut self,
        entity_id: &Entity,
        new_component: T,
        active: bool,
    ) {
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
            error!(
                "No {} for {} with get_mut_or_default. Generating component...",
                T::type_name(),
                index,
            );
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
            error!(
                "No {} for {} with get_mut. Generating component...",
                T::type_name(),
                index
            );

            self.set_component(index, T::default());
        }

        self.get(index).unwrap()
    }

    pub fn component_inspector_raw<F>(
        &mut self,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        entity: &Entity,
        prefab_hashmap: &PrefabMap,
        ui: &mut Ui<'_>,
        is_open: bool,
        mut f: F,
    ) where
        F: FnMut(&mut T, InspectorParameters<'_, '_>),
    {
        if let Some(comp) = self.get_mut(entity) {
            let delete_component = {
                let mut delete = false;
                let name = super::imgui_system::typed_text_ui::<T>();

                ui.tree_node(&imgui::ImString::new(&name))
                    .default_open(true)
                    .frame_padding(false)
                    .build(|| {
                        // COMPONENT INFO
                        let mut comp_info = comp.construct_component_info();
                        super::imgui_system::component_name_and_status(&name, ui, &mut comp_info);
                        comp.take_component_info(&comp_info);

                        // DELETE ENTITY
                        if comp_info.is_deleted {
                            delete = true;
                        } else {
                            let inspector_parameters = InspectorParameters {
                                is_open,
                                uid: &format!("{}{}", comp.entity_id(), &T::type_name()),
                                ui,
                                entities,
                                entity_names,
                                prefabs: prefab_hashmap,
                            };
                            f(comp.inner_mut(), inspector_parameters);
                        }
                    });

                delete
            };
            if delete_component {
                self.unset(entity);
            }
        }
    }
}

pub struct ComponentInfo {
    pub is_active: bool,
    pub is_deleted: bool,
}

impl ComponentInfo {
    pub fn new(is_active: bool, is_deleted: bool) -> Self {
        Self {
            is_active,
            is_deleted,
        }
    }
}
