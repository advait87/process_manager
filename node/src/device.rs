use crate::{helpers::get_hash_string, process::{ GithubRepo, Process, ProcessError, ProcessRequirements }};
use reqwest;
use sha2::{ Sha256, Digest };
use sysinfo::System;
use serde::{ Deserialize, Serialize };
use std::{ env::var, fs::{File, OpenOptions}, io::Write, path::Path, process::{ Command, ExitStatus } };
use zip::ZipArchive;

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
    // I'm again changing the process. The node will fetch a git repo that contains a Dockerfile.
    // It'll use that to build & run the image
    // pub async fn run_process(&mut self, _process: Process) -> Result<(), ProcessError> {
    pub async fn run_process(&self, github_repo: GithubRepo) {
        // https://github.com/Metis-IITGandhinagar/insiit-backend-rust/archive/refs/heads/master.zip
        let url = format!("https://github.com/{}/{}/archive/refs/heads/{}.zip", github_repo.username, github_repo.repo_name, github_repo.branch);
        // Take this from an env variabld
        let output_path = var("GITHUB_REPO_DOWNLOAD_PATH").expect("No path specified to save the github repo");
        let response = match reqwest::get(url).await {
            Ok(r) => r,
            Err(e) => {
                println!("[ERROR] Failed fetching repo: {}", e);
                return
                // return Err(ProcessError::FailedToCreateFile);
            }
        };
        let bytes = match response.bytes().await{
            Ok(b) => b,
            Err(e) => {
                println!("[ERROR] Failed converting reponse to bytes: {}", e);
                return
                // return Err(ProcessError::FailedToCreateFile);
            }
        };
        let zip_file_name = get_hash_string(format!("{}/{}/{}", github_repo.username, github_repo.repo_name, github_repo.branch));
        let zip_file_path = format!("{}/{}.zip", output_path, zip_file_name);

        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&zip_file_path) {
                Ok(f) => f,
                Err(e) => {
                    println!("[ERROR] Failed to create zip file: {}", e);
                    return
                }
            };
        match file.write_all(&bytes) {
            Ok(_) => {},
            Err(e) => println!("[ERROR] Failed to write to zip file: {}", e)
        }

        let target_dir = format!("{}/{}", output_path, zip_file_name);
        let target_dir_path = Path::new(&target_dir);
        let mut archive = match ZipArchive::new(file) {
            Ok(z) => z,
            Err(e) => {
                println!("[ERROR] Failed to validate zip file: {}", e);
                return
                // return Err(ProcessError::FailedToCreateFile);
            }
        };
        match archive.extract(target_dir_path) {
            Ok(_) => {},
            Err(e) => {
                println!("[ERROR] Failed to extract zip file: {}", e);
                // return Err(ProcessError::FailedToCreateFile);
            }
        }

        match Command::new("docker")
            .arg("build")
            .arg("-f")
            .arg(format!("{}/{}-{}/Dockerfile", target_dir, github_repo.repo_name, github_repo.branch))
            .arg("-t")
            .arg(format!("{}/{}/{}", github_repo.username, github_repo.repo_name, github_repo.branch))
            .arg(format!("{}/{}-{}", target_dir, github_repo.repo_name, github_repo.branch))
            .output() {
                Ok(output) => {
                    match output.status.success() {
                        true => {
                            // TODO Just check once if the stdout/stderr are always utf8
                            println!("[INFO] Successfully created docker image with stdout: {}", String::from_utf8(output.stdout).unwrap());
                        },
                        false => {
                            println!("[INFO] Failed to create docker image with stderr: {}", String::from_utf8(output.stderr).unwrap());
                            // return Err(ProcessError::FailedToCreateFile);
                        }
                    }
                },
                Err(e) => {
                    println!("[ERROR] Failed to execute docker build command: {}", e);
                    // return Err(ProcessError::FailedToCreateFile);
                }
            }
        match Command::new("docker")
            .arg("run")
            .arg("-p")
            .arg("4761:4761")
            .arg("-d")
            .arg(format!("{}/{}/{}", github_repo.username, github_repo.repo_name, github_repo.branch))
            .output() {
                Ok(output) => {
                    match output.status.success() {
                        true => {
                            println!("[INFO] Successfully ran the docker container with stdout:\n{}", String::from_utf8(output.stdout).unwrap());
                        },
                        false => {
                            // I don't know if this is a possible output path, cause if a
                            // docker image fails to run, it should go in the Err(e) path of
                            // the outer match, and if the inner code fails, it'll still be
                            // error 0 for the docker run command
                            println!("[INFO] Failed to run the docker container with stdout:\n{}", String::from_utf8(output.stderr).unwrap());
                        }
                    }
                },
                Err(e) => {
                    println!("[ERROR] Failed to execute docker run command: {}", e);
                    // return Err(ProcessError::FailedToCreateFile);
                }
            }
        // Ok(())
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


