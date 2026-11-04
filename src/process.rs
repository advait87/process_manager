use crate::device;

pub struct Process {
    pub argument: i32,
    pub process_function: fn(i32) -> i32,
    pub process_requirements: ProcessRequirements
}

pub struct ProcessRequirements {
    pub memory_in_megabytes: u64,
    pub cores: usize,
}

pub enum ProcessError {
    NotEnoughAllowedMaxMemory,
    NotEnoughAllowedMaxCoreCount,
    NotEnoughAvailableMemory,
    NotEnoughAvailableCores
}
