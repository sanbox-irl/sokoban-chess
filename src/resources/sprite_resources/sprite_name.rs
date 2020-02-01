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
pub enum SpriteName {
    WhitePixel,
    Background,
    PixelMainCharacterWalking,
    PixelMainCharacterStanding,
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
