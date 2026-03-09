use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod directions {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::Direction;

    pub fn serialize<S>(value: &Vec<Direction>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Direction::vec_to_string(value).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Direction>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Direction::try_vec_from_string(String::deserialize(deserializer)?)
            .map_err(|e| serde::de::Error::custom(format!("{}", e)))
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
