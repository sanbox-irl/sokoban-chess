use super::{
    cardinals::{CardinalPrime, FacingHorizontal},
    hardware_interfaces::KeyboardInput,
};
use winit::event::VirtualKeyCode as VK;

#[derive(Debug, Default)]
pub struct ActionMap {
    move_direction: Option<CardinalPrime>,
    switch_active_player: Option<FacingHorizontal>,
}

impl ActionMap {
    pub fn update(&mut self, kb: &KeyboardInput) {
        *self = ActionMap::default();

        for key in kb.pressed_keys.iter() {
            self.move_direction = match key {
                VK::Left | VK::A => Some(CardinalPrime::Left),
                VK::Right | VK::D => Some(CardinalPrime::Right),
                VK::Down | VK::S => Some(CardinalPrime::Down),
                VK::Up | VK::W => Some(CardinalPrime::Up),
                _ => None,
            }
        }

        if kb.is_pressed(VK::Q) {
            self.switch_active_player = Some(FacingHorizontal::Left);
        }

        if kb.is_pressed(VK::E) {
            self.switch_active_player = Some(FacingHorizontal::Right);
        }
    }

    pub fn move_direction(&self) -> Option<CardinalPrime> {
        self.move_direction
    }

    pub fn switch_active_player(&self) -> Option<FacingHorizontal> {
        self.switch_active_player
    }
}
