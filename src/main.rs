#![allow(unused)]
use sha2::{ Digest, Sha256 };
use sysinfo::{ System, CpuRefreshKind, RefreshKind };
mod device;
mod process;


fn main() {
    // match System::new_all() {
    //     Ok(x) => {
    //         let available_ram_in_megabytes = x.free_memory()/1024/1024;
    //         println!("There's {available_ram_in_megabytes} MBs free RAM on this machine");
    //         if available_ram_in_megabytes > 100 {
    //             println!("There's enough free memory to run a process")
    //         } else {
    //             println!("There isn't enough memory free to run a process")
    //         }
    //     },
    //     Err(e) => println!("Error: {:?}", e)
    // }
    // match System::physical_core_count() {
    //     Ok(core_count) => println!("There are {core_count} cores on this machine"),
    //     Err(e) => println!("Couldn't fetch the core count")
    // }
    let process = process::Process {
        process_function: |a: i32| 2*a,
        argument: 100,
        process_requirements: process::ProcessRequirements { memory_in_megabytes: 10, cores: 1 }
    };

    let mut device = device::Device {
        hash: Sha256::digest("hi").into(),
        device_info: device::DeviceInfo::new(None, None).unwrap()
    };

    println!("The output for the process is: {:?}", device.run_process(process))
}
