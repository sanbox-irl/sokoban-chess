use strum_macros::Display;

#[derive(Debug, Display, PartialEq, Ord, PartialOrd, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum SceneName {
    Main,
}
