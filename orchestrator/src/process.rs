use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessRequirements {
    pub memory_in_megabytes: u64,
    pub cores: usize,
}
