
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, Display};

#[derive(Debug, Clone, Copy, EnumIter, Display, PartialEq)]
#[repr(usize)]
pub enum BrickAssets {
    MicroBrick,
    MicroWedge,
    MicroWedgeCorner,
    MicroWedgeTriangleCorner,
    MicroWedgeOuterCorner,
    MicroWedgeInnerCorner,
}
impl BrickAssets {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn prefix(self) -> &'static str {
        if self.index() <= 3 {"PB_Default"} else {""}
    }
    pub fn name(self) -> String {
        format!("{}{}", self.prefix(), self.to_string())
    }
    pub fn names() -> Vec<String> {
        Self::iter().map(|asset| asset.name()).collect()
    }
}