use serde::{Deserialize, Deserializer};

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

pub fn username<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;
    let len = s.len();
    if len < 3 || len > 20 {
        Err(serde::de::Error::custom("username must be between 3 and 20 characters"))
    } else if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Err(serde::de::Error::custom("username can only consist of a-z, 0-9 and _"))
    } else {
        s.make_ascii_lowercase();
        Ok(s)
    }
}

pub fn password<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let len = s.len();
    if len < 8 || len > 128 {
        Err(serde::de::Error::custom("password must be between 8 and 128 characters"))
    } else if !s.chars().all(|c| !c.is_control()) {
        Err(serde::de::Error::custom("password contains illegal characters"))
    } else {
        Ok(s)
    }
}
 