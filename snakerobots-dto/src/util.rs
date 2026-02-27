use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod moves {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::GameMoveDto;

    pub fn serialize<S>(value: &Vec<GameMoveDto>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value
            .iter()
            .map(|m| char::from(*m))
            .collect::<String>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<GameMoveDto>, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .chars()
            .map(|c| {
                GameMoveDto::try_from(c)
                    .map_err(|_| serde::de::Error::custom(format!("invalid move: '{}'", c)))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

pub trait Hex: Sized {
    fn to_hex(&self) -> String;
    fn from_hex(str: &str) -> Option<Self>;

    fn serialize<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.to_hex().serialize(serializer)
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::from_hex(&String::deserialize(deserializer)?)
            .ok_or_else(|| serde::de::Error::custom("invalid hex"))
    }
}

impl Hex for u64 {
    fn to_hex(&self) -> String {
        format!("{:016x}", self)
    }

    fn from_hex(str: &str) -> Option<Self> {
        Self::from_str_radix(str, 16).ok()
    }
}
