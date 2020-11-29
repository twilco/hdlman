use strum::{EnumIter, IntoEnumIterator};

pub const SUPPORTED_TARGETS: [SupportedEntity; 1] = [SupportedEntity {
    name: "ecp5-85k",
    description: indoc! {"
            The 85k LUT variant of the Lattice ECP5 chip.
            https://www.latticesemi.com/Products/FPGAandCPLD/ECP5
        "},
}];

pub const SUPPORTED_DEV_BOARDS: [SupportedEntity; 1] = [SupportedEntity {
    name: "ulx3s",
    description: indoc! {"
            The ULX3s dev-board made by Radiona.
            https://radiona.org/ulx3s/
        "},
}];

/// Behavior surrounding resources associated with an entity, e.g. an LPF file for a dev-board.
pub trait ResourceAssociation {
    fn associated_resources(&self) -> Option<Vec<Resource>>;
}

#[derive(Clone, Debug)]
pub struct SupportedEntity<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub filename: String,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum Target {
    ECP5_85k,
}

impl ResourceAssociation for Target {
    fn associated_resources(&self) -> Option<Vec<Resource>> {
        match self {
            Target::ECP5_85k => None,
        }
    }
}

impl AsRef<str> for Target {
    fn as_ref(&self) -> &str {
        match self {
            Target::ECP5_85k => "ecp5-85k",
        }
    }
}

#[derive(Clone, Debug)]
pub struct StrParseError {
    received: String,
}

impl std::str::FromStr for Target {
    type Err = StrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for target in Target::iter() {
            if s == target.as_ref() {
                return Ok(target);
            }
        }

        Err(StrParseError {
            received: s.to_owned(),
        })
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum DevBoard {
    ULX3S,
}

impl ResourceAssociation for DevBoard {
    fn associated_resources(&self) -> Option<Vec<Resource>> {
        match self {
            DevBoard::ULX3S => Some(vec![Resource {
                filename: "ulx3s_v20.lpf".to_owned(),
            }]),
        }
    }
}

impl AsRef<str> for DevBoard {
    fn as_ref(&self) -> &str {
        match self {
            DevBoard::ULX3S => "ulx3s",
        }
    }
}

impl std::str::FromStr for DevBoard {
    type Err = StrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for board in DevBoard::iter() {
            if s == board.as_ref() {
                return Ok(board);
            }
        }

        Err(StrParseError {
            received: s.to_owned(),
        })
    }
}
