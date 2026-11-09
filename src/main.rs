#![allow(unused)]
use sysinfo::{ System, CpuRefreshKind, RefreshKind };
mod device;
mod process;


fn main() {
    match System::new_all() {
        Ok(x) => {
            let available_ram_in_megabytes = x.free_memory()/1024/1024;
            println!("There's {available_ram_in_megabytes} MBs free RAM on this machine");
            if available_ram_in_megabytes > 100 {
                println!("There's enough free memory to run a process")
            } else {
                println!("There isn't enough memory free to run a process")
            }
        },
        Err(e) => println!("Error: {:?}", e)
    }
    match System::physical_core_count() {
        Ok(core_count) => println!("There are {core_count} cores on this machine"),
        Err(e) => println!("Couldn't fetch the core count")
    }
}
