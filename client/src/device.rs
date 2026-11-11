use crate::process:: { Process, ProcessError, ProcessRequirements };
use sha2::{ Sha256, Digest };
use sysinfo::System;
use serde::{ Deserialize, Serialize };

pub struct DeviceInfo {
    pub device_system: System,
    pub total_memory_in_megabytes: u64,
    pub physical_core_count: usize,

    pub allowed_max_memory_in_megabytes: u64,
    pub allowed_max_core_count: usize
}


impl DeviceInfo {
    pub fn new(allowed_max_memory_in_megabytes: Option<u64>, allowed_max_core_count: Option<usize>) -> Result<Self, ()> {
        let physical_core_count = match System::physical_core_count() {
            Ok(p) => p,
            Err(e) => return Err(())
        };
        match System::new() {
            Ok(mut s) => {
                s.refresh_all();
                return Ok(DeviceInfo {
                    // For pleasing the compiler, I've put the device_system: s at the end, though
                    // it's not what I like. But it's a little out of my depth, will solve later
                    // TODO read above comment
                    total_memory_in_megabytes: s.total_memory()/1024/1024,
                    allowed_max_memory_in_megabytes: allowed_max_memory_in_megabytes.unwrap_or_else(|| {
                        let m = s.available_memory()/1024/1024;
                        println!("[INFO] allowed_max_memory_in_megabytes not specified. Setting it to: {}", m);
                        m
                    }),
                    allowed_max_core_count: allowed_max_core_count.unwrap_or_else(|| {
                        println!("[INFO] allowed_max_core_count not specified. Setting it to: {}", physical_core_count);
                        physical_core_count
                    }),
                    device_system: s,
                    physical_core_count,
                })
            },
            Err(e) => return Err(())
        }
        Err(())
    }

    pub fn available_memory_in_megabytes(&mut self) -> u64 {
        self.device_system.refresh_memory();
        return self.device_system.available_memory()/1024/1024
    }

    // TODO
    pub fn available_cores(&mut self) -> usize {
        self.device_system.refresh_all();
        return 1
    }


    pub fn check_requirements(&mut self, _process: &Process) -> Result<(), ProcessError> {
        self.device_system.refresh_all();
        if self.allowed_max_core_count < _process.process_requirements.cores {
            return Err(ProcessError::NotEnoughAllowedMaxCoreCount)
        } else if self.allowed_max_memory_in_megabytes < _process.process_requirements.memory_in_megabytes {
            return Err(ProcessError::NotEnoughAllowedMaxMemory)
        } else if self.available_cores() < _process.process_requirements.cores {
            return Err(ProcessError::NotEnoughAvailableCores)
        } else if self.available_memory_in_megabytes() < _process.process_requirements.memory_in_megabytes {
            return Err(ProcessError::NotEnoughAvailableMemory)
        }
        Ok(())
    }
}

pub struct Device {
    pub hash: [u8; 32],
    // maybe update this to make it a reference to DeviceInfo if needed
    pub device_info: DeviceInfo
}


impl Device {
    // The reason why process_function is abstracted into run_process is cause run_process will
    // handle and return ProcessError if it happens - out of resources, runtime error, etc
    pub fn run_process(&mut self, _process: Process) -> Result<i32, ProcessError> {
        match self.device_info.check_requirements(&_process) {
            Ok(()) => Ok((_process.process_function)(_process.argument)),
            Err(e) => Err(e)
        }
    }

    pub fn public_info(&mut self) -> DevicePublicInfo {
        DevicePublicInfo {
            hash: self.hash,
            allowed_max_core_count: self.device_info.allowed_max_core_count,
            allowed_max_memory_in_megabytes: self.device_info.allowed_max_memory_in_megabytes,
            available_cores: self.device_info.available_cores(),
            available_memory_in_megabytes: self.device_info.available_memory_in_megabytes()
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DevicePublicInfo {
    pub hash: [u8; 32],
    pub available_memory_in_megabytes: u64,
    pub available_cores: usize,
    pub allowed_max_memory_in_megabytes: u64,
    pub allowed_max_core_count: usize
}

impl DevicePublicInfo {
    pub fn check_requirements(&self, process_requirements: &ProcessRequirements) -> Result<(), ProcessError> {
        if self.allowed_max_core_count < process_requirements.cores {
            return Err(ProcessError::NotEnoughAllowedMaxCoreCount)
        } else if self.allowed_max_memory_in_megabytes < process_requirements.memory_in_megabytes {
            return Err(ProcessError::NotEnoughAllowedMaxMemory)
        } else if self.available_cores < process_requirements.cores {
            return Err(ProcessError::NotEnoughAvailableCores)
        } else if self.available_memory_in_megabytes < process_requirements.memory_in_megabytes {
            return Err(ProcessError::NotEnoughAvailableMemory)
        }
        Ok(())
    }
}


