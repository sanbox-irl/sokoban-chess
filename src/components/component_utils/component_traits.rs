use super::{ComponentList, Entity, Name, SerializedEntity};

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
}

impl<T: ComponentBounds + std::fmt::Debug + typename::TypeName + 'static> ComponentListBounds
    for ComponentList<T>
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
