use sha2::{ Sha256, Digest };
use sysinfo::System;
use crate::process:: { Process, ProcessError };

pub struct DeviceInfo {
    pub device_system: System,
    pub total_memory_in_megabytes: u64,
    pub physical_core_count: usize,

    pub allowed_max_memory_in_megabytes: u64,
    pub allowed_max_core_count: usize
}

pub struct Device {
    pub hash: [u8; 32],
    // maybe update this to make it a reference to DeviceInfo if needed
    pub device_info: DeviceInfo
}

impl Device {
    fn run_process(&self, _process: Process) -> Result<i32, ProcessError> {
        if self.device_info.allowed_max_core_count < _process.process_requirements.cores {
            return Err(ProcessError::NotEnoughAllowedMaxCoreCount)
        } else if self.device_info.allowed_max_memory_in_megabytes < _process.process_requirements.memory_in_megabytes {
            return Err(ProcessError::NotEnoughAllowedMaxMemory)
        } else if self.device_info.available_cores() < _process.process_requirements.cores {
            return Err(ProcessError::NotEnoughAvailableCores)
        } else if self.device_info.available_memory_in_megabytes() < _process.process_requirements.memory_in_megabytes {
            return Err(ProcessError::NotEnoughAvailableMemory)
        }
        Ok((_process.process_function)(_process.argument))
    }
}

impl DeviceInfo {
    fn new(allowed_max_memory_in_megabytes: Option<u64>, allowed_max_core_count: Option<usize>) -> Result<Self, ()> {
        let physical_core_count = match System::physical_core_count() {
            Ok(p) => p,
            Err(e) => return Err(())
        };
        match System::new() {
            Ok(s) => {
                return Ok(DeviceInfo {
                    // For pleasing the compiler, I've put the device_system: s at the end, though
                    // it's not what I like. But it's a little out of my depth, will solve later
                    // TODO read above comment
                    total_memory_in_megabytes: s.total_memory()/1024/1024,
                    allowed_max_memory_in_megabytes: allowed_max_memory_in_megabytes.unwrap_or_else(|| s.available_memory()/1024/1024),
                    allowed_max_core_count: allowed_max_core_count.unwrap_or_else(|| physical_core_count),
                    device_system: s,
                    physical_core_count,
                })
            },
            Err(e) => return Err(())
        }
        Err(())
    }

    fn available_memory_in_megabytes(&self) -> u64 {
        return self.device_system.available_memory()/1024/1024
    }

    // TODO
    fn available_cores(&self) -> usize {
        return 1
    }

}
