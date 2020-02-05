use super::Color;

pub struct NameInspectorResult {
    pub serialize_name: Option<String>,
    pub unserialize: bool,
    pub inspect: bool,
    pub show_children: bool,
    pub clone: bool,
    pub delete: bool,
}

impl Default for NameInspectorResult {
    fn default() -> Self {
        Self {
            show_children: true,
            serialize_name: None,
            unserialize: false,
            inspect: false,
            clone: false,
            delete: false,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct NameInspectorParameters {
    pub has_children: bool,
    pub is_prefab: bool,
    pub depth: usize,
    pub being_inspected: bool,
    pub is_serialized: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityListInformation {
    pub open: bool,
    pub color: Color,
    pub edit_name: NameEdit,
}

impl Default for EntityListInformation {
    fn default() -> Self {
        EntityListInformation {
            open: false,
            color: Color::WHITE,
            edit_name: NameEdit::NoEdit,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NameEdit {
    First,
    Editing,
    NoEdit,
}
