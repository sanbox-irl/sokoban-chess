use strum_macros::{Display, EnumCount, EnumIter, EnumString};

#[derive(
    Debug,
    Display,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    EnumIter,
    EnumString,
    EnumCount,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
#[strum(serialize_all = "snake_case")]
pub enum SpriteName {
    Zelda,
    Link,
    CenterDot,
    KennyPlayerDown,
    KennyPlayerLeft,
    KennyPlayerRight,
    KennyPlayerUp,
    WhitePixel,
    PixelGreenFloors,
    PixelGreenWalls,
    PixelMainCharacterWalking,
    PixelMainCharacterStanding,
    NormalNPC,
    BangUI,
    HookUI,
    SpeechBubble
}

impl SpriteName {
    pub fn better_display(&self) -> String {
        format!("{:?}", self)
    }
}

impl Default for SpriteName {
    fn default() -> Self {
        Self::WhitePixel
    }
}
