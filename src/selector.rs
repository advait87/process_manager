use crate::{ device, helpers, process };
use sysinfo::System;

#[derive(Debug, PartialEq)]
pub enum SelectNodeError {
    NoDevicesPresent,
    NoDeviceWithRequiredMemory,
    NoDeviceWithRequiredCoreCount
}

pub fn select_node<'a>(devices: &'a Vec<device::DevicePublicInfo>, _process: &process::Process) -> Result<&'a device::DevicePublicInfo, SelectNodeError> {
    // For now, let's say devices vec is not sorted. Later we'll need to use a data structure such
    // that it's sorted by RAM as well as core count. Maybe 2 different sorted vecs using red black
    // trees idk. But problem for another day. rn linear search
    // TODO read above comment
    for mut device in devices {
        match device.check_requirements(_process) {
            Ok(()) => return Ok(device),
            Err(_) => {}
        }
    }
    return Err(SelectNodeError::NoDevicesPresent)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use super::*;

    // #[test]
    // Big blunder - I tried writing table driven tests style of tests here, but this is more of an
    // aggregator test situation, so will need to add separate tests.
    // #[test_case(device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 2, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 }, process::Process { process_function: |a: i32| a*a, argument: 6, process_requirements: process::ProcessRequirements { memory_in_megabytes: 10, cores: 1 } }, Err(SelectNodeError::NoDeviceWithRequiredMemory))]
    // fn test_select_node(_device: device::DevicePublicInfo, _process: process::Process, expected_output: Result<device::DevicePublicInfo, SelectNodeError>) {
        // assert_eq!(_device.check_requirements(&_process), expected_output)
        // let devices: Vec<device::DevicePublicInfo> = vec![
        //     device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 2, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
        //     device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 5, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
        //     device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 0, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
        //     device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 1, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
        //     device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 1, allowed_max_core_count: 2, available_memory_in_megabytes: 111, allowed_max_memory_in_megabytes: 25 },
        // ];
    // }

    #[test]
    fn test_select_node() {
        let devices: Vec<device::DevicePublicInfo> = vec![
            device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 2, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
            device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 5, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
            device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 0, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
            device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 1, allowed_max_core_count: 2, available_memory_in_megabytes: 96, allowed_max_memory_in_megabytes: 256 },
            device::DevicePublicInfo { hash: helpers::get_random_sha256(), available_cores: 1, allowed_max_core_count: 2, available_memory_in_megabytes: 111, allowed_max_memory_in_megabytes: 25 },
        ];

        let process = process::Process {
            process_function: |a: i32| a*a,
            argument: 6,
            process_requirements: process::ProcessRequirements { memory_in_megabytes: 10, cores: 1 }
        };

        let selected_device = select_node(&devices, &process);
        assert_eq!(selected_device, Ok(&devices[0]))
    }
}
