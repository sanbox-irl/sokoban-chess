use super::{imgui_system, ComponentBounds, InspectorParameters, TweenActivation, TweenRepeatOnPlay, Vec2};
use simple_tween::{Ease, Tweener};

#[derive(Debug, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct TweenTransform {
    pub ease: Ease,
    pub distance: Vec2,
    pub duration: f32,
    pub activation: TweenActivation,
    pub repeat: TweenRepeatOnPlay,
    pub transform_original: Vec2,

    #[serde(skip)]
    tweeners: Option<(Tweener, Tweener)>,
}

impl TweenTransform {
    pub fn create_tweener(&mut self, original_position: Vec2) {
        self.transform_original = original_position;

        self.tweeners = Some((
            Tweener::new(
                original_position.x,
                original_position.x + self.distance.x,
                self.duration,
                self.ease,
            ),
            Tweener::new(
                original_position.y,
                original_position.y + self.distance.y,
                self.duration,
                self.ease,
            ),
        ));
    }
}

impl ComponentBounds for TweenTransform {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        // pub ease: Ease,
        if let Some(new_ease) = imgui_system::typed_enum_selection(ip.ui, &self.ease, ip.uid) {
            self.ease = new_ease;

            if let Some((x_tween, y_tween)) = &mut self.tweeners {
                x_tween.ease = self.ease;
                y_tween.ease = self.ease;
            }
        }

        // pub distance: Vec2,
        self.distance
            .inspector(&ip.ui, &imgui::im_str!("Distance##{}", ip.uid));

        // pub duration: f32,
        ip.ui
            .drag_float(&imgui::im_str!("Duration##{}", ip.uid), &mut self.duration)
            .build();

        // pub activation: TweenActivation,
        if let Some(activation) = imgui_system::typed_enum_selection(ip.ui, &self.activation, ip.uid) {
            self.activation = activation;
        }

        // pub repeat: TweenRepeatOnPlay,
        if let Some(new_repeat) = imgui_system::typed_enum_selection(ip.ui, &self.repeat, ip.uid) {
            self.repeat = new_repeat;
        }
    }
}

impl PartialEq for TweenTransform {
    fn eq(&self, other: &TweenTransform) -> bool {
        self.ease == other.ease
            && self.distance == other.distance
            && self.activation == other.activation
            && self.repeat == other.repeat
    }
}

impl Clone for TweenTransform {
    fn clone(&self) -> Self {
        TweenTransform {
            transform_original: self.transform_original,
            tweeners: None,
            duration: self.duration,
            ease: self.ease,
            distance: self.distance,
            activation: self.activation,
            repeat: self.repeat,
        }
    }
}
