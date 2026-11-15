use crate::device;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize)]
pub struct Process {
    pub python_code: String,
    pub process_requirements: ProcessRequirements
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessRequirements {
    pub memory_in_megabytes: u64,
    pub cores: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ProcessError {
    FailedToCreateFile,
    NotEnoughAllowedMaxMemory,
    NotEnoughAllowedMaxCoreCount,
    NotEnoughAvailableMemory,
    NotEnoughAvailableCores
}
