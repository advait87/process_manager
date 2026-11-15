use crate:: { process:: { Process, ProcessError, ProcessRequirements }, helpers::python_docker_file_string };
use sha2::{ Sha256, Digest };
use sysinfo::System;
use serde::{ Deserialize, Serialize };
use std::{ fs::File, io::Write, path::Path, process::ExitStatus };

#[derive(Debug)]
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
                    device_system: match true {
                        true => {
                            println!("yellow");
                            s
                        },
                        false => s
                    },
                    physical_core_count: match true {
                        true => {
                            println!("green");
                            physical_core_count
                        },
                        false => physical_core_count
                    },
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

#[derive(Debug)]
pub struct Device {
    pub hash: [u8; 32],
    // maybe update this to make it a reference to DeviceInfo if needed
    pub device_info: DeviceInfo
}


impl Device {
    // The reason why process_function is abstracted into run_process is cause run_process will
    // handle and return ProcessError if it happens - out of resources, runtime error, etc
    // Okay, so I'm changing run_process. It'll instead of running a native rust function, it'll run
    // python/JS code in a sandbox, and return a signal that the process has started or return an
    // error
    pub fn run_process(&mut self, _process: Process) -> Result<(), ProcessError> {
        match self.device_info.check_requirements(&_process) {
            Ok(()) => {
                let python_file_path = Path::new("/tmp/process_manager/python_code.py");
                // spin up a docker image, for now it'll be for python, and execute the python file
                // in that docker container
                let file_path = Path::new("/tmp/process_manager/tmpfile");
                println!("Creating docker file: {}", file_path.to_str().unwrap().to_string());
                let file = match File::create(file_path) {
                    Ok(mut file) => {
                        println!("Created file");
                        let docker_file_string = python_docker_file_string(python_file_path);
                        match file.write_all(docker_file_string.as_bytes()) {
                            Ok(_) => { println!("Wrote to docker file") },
                            Err(e) => {
                                println!("Failed to create docker file: {}", e);
                                return Err(ProcessError::FailedToCreateFile)
                            }
                        }
                        file
                    },
                    Err(e) => {
                        println!("couldn't create docker file: {}", e);
                        return Err(ProcessError::FailedToCreateFile);
                    }

                };


                let python_file = match File::create(python_file_path) {
                    Ok(mut python_file) => {
                        println!("Created file");
                        match python_file.write_all(_process.python_code.as_bytes()) {
                            Ok(_) => { println!("Wrote to python file") },
                            Err(e) => {
                                println!("Failed to create python file: {}", e);
                                return Err(ProcessError::FailedToCreateFile)
                            }
                        }
                        file
                    },
                    Err(e) => {
                        println!("couldn't create python file: {}", e);
                        return Err(ProcessError::FailedToCreateFile);
                    }

                };

                // TODO WARN don't unwrap, handle
                println!("Building from file: {}", file_path.to_str().unwrap().to_string());
                println!("Running command: {}", format!("docker build -f {} -t tmpimage /tmp/process_manager", file_path.to_str().unwrap().to_string()));
                let image_build_output = match std::process::Command::new("docker")
                    .arg("build")
                    .arg("-f")
                    .arg(file_path)
                    .arg("-t")
                    .arg("tmpimage")
                    .arg("/tmp/process_manager")
                    .output() {
                        Ok(output) => {
                            println!("Executed docker build command");
                            match output.status.success() {
                                true => {
                                    // TODO WARN don't unwrap, handle
                                    println!("The process returned output: {}", String::from_utf8(output.stdout).unwrap());
                                },
                                false => {
                                    // TODO WARN don't unwrap, handle
                                    println!("The process failed with output: {}", String::from_utf8(output.stderr).unwrap());
                                }
                            }
                        },
                        Err(e) => {
                            println!("Couldn't execute build command: {}", e);

                        }
                    };
                let final_output = match std::process::Command::new("docker")
                    .arg("run")
                    .arg("tmpimage")
                    .output() {
                        Ok(output) => {
                            println!("Executed docker run command");
                            match output.status.success() {
                                true => {
                                    // TODO WARN don't unwrap, handle
                                    println!("The process returned output: {}", String::from_utf8(output.stdout).unwrap());
                                },
                                false => {
                                    // TODO WARN don't unwrap, handle
                                    println!("The process failed with output: {}", String::from_utf8(output.stderr).unwrap());
                                }
                            }
                        },
                        Err(e) => {
                            println!("The run command failed: {}", e);
                        }
                    };
                Ok(())
            },
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


