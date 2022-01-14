use std::convert::TryInto;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: &'static str,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct World {
    pub id: i32,
    pub randomnumber: i32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct Fortune {
    pub id: i32,
    pub message: String,
}

// The ids are stored in MongoDB as floating point numbers, so need a custom deserialization implementation
// to handle converting them.
impl<'de> Deserialize<'de> for Fortune {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct FloatFortune {
            id: f32,
            message: String
        }

        let float = FloatFortune::deserialize(deserializer)?;
        Ok(Fortune {
            id: float.id as i32,
            message: float.message
        })
    }
}

impl Default for Fortune {
    fn default() -> Self {
        Fortune {
            id: -1,
            message: "".to_string()
        }
    }
}
