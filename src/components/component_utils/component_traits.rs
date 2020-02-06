use super::{Component, ComponentList, Entity, Name, SerializedEntity};
use imgui::Ui;

pub trait ComponentBounds {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>);
}

pub trait ComponentSerializedBounds {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>);
}

pub struct InspectorParameters<'a, 'b> {
    pub ui: &'b mut imgui::Ui<'a>,
    pub entities: &'b [Entity],
    pub entity_names: &'b ComponentList<Name>,
    pub prefabs: &'b std::collections::HashMap<uuid::Uuid, SerializedEntity>,
    pub uid: &'b str,
    pub is_open: bool,
}

pub trait ComponentListBounds {
    fn expand_list(&mut self);
    fn unset(&mut self, index: &Entity) -> bool;
    fn dump_to_log(&self, index: &Entity);
    fn clone_entity(&mut self, index: &Entity, new_entity: &Entity);
    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>);
    fn component_inspector(
        &mut self,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        entity: &Entity,
        prefab_hashmap: &std::collections::HashMap<uuid::Uuid, SerializedEntity>,
        ui: &mut imgui::Ui<'_>,
        is_open: bool,
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
            self.set(new_entity, Component::new(new_entity, new_component));
        }
    }

    fn component_add_button(&mut self, index: &Entity, ui: &imgui::Ui<'_>) {
        if imgui::MenuItem::new(&imgui::ImString::new(
            super::imgui_system::typed_text_ui::<T>(),
        ))
        .enabled(self.get(index).is_none())
        .build(ui)
        {
            self.set(index, Component::new(index, T::default()));
        }
    }

    fn component_inspector(
        &mut self,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        entity: &Entity,
        prefab_hashmap: &std::collections::HashMap<uuid::Uuid, SerializedEntity>,
        ui: &mut Ui<'_>,
        is_open: bool,
    ) {
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
                            comp.inner_mut().entity_inspector(inspector_parameters);
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
