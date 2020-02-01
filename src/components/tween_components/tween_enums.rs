use strum_macros::EnumIter;

#[derive(
    Debug, PartialEq, Ord, PartialOrd, Eq, Copy, Clone, EnumIter, Serialize, Deserialize, typename::TypeName,
)]
#[strum(serialize_all = "snake_case")]
pub enum TweenActivation {
    Code,
    OnCreation,
}

#[derive(
    Debug, PartialEq, Ord, PartialOrd, Eq, Copy, Clone, EnumIter, Serialize, Deserialize, typename::TypeName,
)]
#[strum(serialize_all = "snake_case")]
pub enum TweenRepeatOnPlay {
    Once(bool),
    Infinite,
}

impl Default for TweenActivation {
    fn default() -> Self {
        TweenActivation::Code
    }
}

impl Default for TweenRepeatOnPlay {
    fn default() -> Self {
        TweenRepeatOnPlay::Once(false)
    }
}
