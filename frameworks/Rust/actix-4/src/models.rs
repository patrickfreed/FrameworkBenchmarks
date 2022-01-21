use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    err: anyhow::Error,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl actix_web::error::ResponseError for Error {}

impl<T> From<T> for Error
where
    T: Into<anyhow::Error>,
{
    fn from(e: T) -> Self {
        Error { err: e.into() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: &'static str,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub id: i32,
    pub random_number: i32,
}

// The ids are stored in MongoDB as floating point numbers, so need a custom deserialization implementation
// to handle converting them.
impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct FloatWorld {
            id: f32,
            random_number: f32,
        }

        let float = FloatWorld::deserialize(deserializer)?;
        Ok(World {
            id: float.id as i32,
            random_number: float.random_number as i32,
        })
    }
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
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct FloatFortune {
            id: f32,
            message: String,
        }

        let float = FloatFortune::deserialize(deserializer)?;
        Ok(Fortune {
            id: float.id as i32,
            message: float.message,
        })
    }
}

impl Default for Fortune {
    fn default() -> Self {
        Fortune {
            id: -1,
            message: "".to_string(),
        }
    }
}

pub struct Queries {
    pub q: usize,
}

impl<'de> Deserialize<'de> for Queries {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct Q {
            q: Option<String>,
        }

        let q = Q::deserialize(deserializer)?;
        let n = match q.q {
            Some(s) => {
                let i: i32 = s.parse().unwrap_or(1);
                std::cmp::max(1, std::cmp::min(500, i)) as usize
            }
            None => 1,
        };
        Ok(Queries { q: n })
    }
}
