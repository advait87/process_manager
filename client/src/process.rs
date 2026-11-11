use crate::device;
use serde::{ Deserialize, Serialize };

#[derive(Debug)]
pub struct Process {
    pub argument: i32,
    pub process_function: fn(i32) -> i32,
    pub process_requirements: ProcessRequirements
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessRequirements {
    pub memory_in_megabytes: u64,
    pub cores: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ProcessError {
    NotEnoughAllowedMaxMemory,
    NotEnoughAllowedMaxCoreCount,
    NotEnoughAvailableMemory,
    NotEnoughAvailableCores
}
