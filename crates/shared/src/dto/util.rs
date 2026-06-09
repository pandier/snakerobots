use validator::ValidationError;

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

pub fn validate_username_chars(value: &str) -> Result<(), ValidationError> {
    if value.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(ValidationError::new("chars"))
    }
}

pub fn validate_non_control_chars(value: &str) -> Result<(), ValidationError> {
    if value.chars().all(|c| !c.is_control()) {
        Ok(())
    } else {
        Err(ValidationError::new("chars"))
    }
}
