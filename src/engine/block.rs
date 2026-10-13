#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub(crate) enum BlockType {
    #[default]
    Dirt,
    Stone,
}

impl BlockType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            BlockType::Dirt => "dirt",
            BlockType::Stone => "stone",
        }
    }
}
