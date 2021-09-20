use std::any;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub enum Commands {
    GetCount,
    GetName,
    MutateCount(Option<i32>),
}

#[derive(Serialize, Deserialize)]
pub struct Payload<T> {
    pub data: T,
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("missing input value")]
    MissingValue,
}

impl Commands {
    pub fn handler_path(self) -> &'static str {
        match self {
            Commands::GetCount => "/count",
            Commands::GetName => "/name",
            Commands::MutateCount(_) => "/count/mutate/:value",
        }
    }

    pub fn query_string(self) -> Result<String, QueryError> {
        match self {
            Commands::MutateCount(val) => val
                .map(|v| {
                    Commands::MutateCount(None)
                        .handler_path()
                        .replace(":value", &v.to_string())
                })
                .ok_or(QueryError::MissingValue),

            _ => Ok(self.handler_path().to_string()),
        }
    }
}
