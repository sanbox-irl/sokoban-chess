use super::{ComponentBounds, InspectorParameters, Vec2Int};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct GridObject {
    pub grid_type: GridType,
    #[serde(skip)]
    pub move_to_point_pos: Vec2Int,
    #[serde(skip)]
    pub move_to_point: bool,
    #[serde(skip)]
    pub register: bool,
}

impl GridObject {
    pub fn grid_type(&self) -> GridType {
        self.grid_type
    }
}

impl ComponentBounds for GridObject {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_grid_type) =
            super::imgui_system::typed_enum_selection(ip.ui, &self.grid_type, ip.uid)
        {
            self.grid_type = new_grid_type;
        }

        if let GridType::Button(pos) = &mut self.grid_type {
            let mut pos_v2: Vec2Int = Vec2Int::new(pos.0 as i32, pos.1 as i32);
            if pos_v2.vec2int_inspector(
                ip.ui,
                &imgui::im_str!("Block Drop Location Offset##{}", ip.uid),
            ) {
                *pos = (pos_v2.x as isize, pos_v2.y as isize);
            }
        }

        if self
            .move_to_point_pos
            .vec2int_inspector(ip.ui, &imgui::im_str!("##Move to Point{}", ip.uid))
        {
            if self.move_to_point_pos.x < 0 {
                self.move_to_point_pos.x = 0;
            }

            if self.move_to_point_pos.y < 0 {
                self.move_to_point_pos.y = 0;
            }
        }

        ip.ui.same_line(0.0);
        if ip
            .ui
            .button(&imgui::im_str!("Move to Point##{}", ip.uid), [0.0, 0.0])
        {
            self.move_to_point = true;
        }

        if ip
            .ui
            .button(&imgui::im_str!("Register Position##{}", ip.uid), [0.0, 0.0])
        {
            self.register = true;
        }
    }
}

#[derive(
    Copy,
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    strum_macros::EnumCount,
    typename::TypeName,
)]
pub enum GridType {
    Player,
    Pushable,
    Blockable,
    NonInteractable,
    Flag,
    Button((isize, isize)),
}

impl Default for GridType {
    fn default() -> GridType {
        GridType::Blockable
    }
}
