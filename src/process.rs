use crate::device;

#[derive(Debug)]
pub struct Process {
    pub argument: i32,
    pub process_function: fn(i32) -> i32,
    pub process_requirements: ProcessRequirements
}

#[derive(Debug)]
pub struct ProcessRequirements {
    pub memory_in_megabytes: u64,
    pub cores: usize,
}

#[derive(Debug)]
pub enum ProcessError {
    NotEnoughAllowedMaxMemory,
    NotEnoughAllowedMaxCoreCount,
    NotEnoughAvailableMemory,
    NotEnoughAvailableCores
}
