#![allow(unused)]

use axum::{
    extract::{ Json, State },
    handler::Handler,
    http::StatusCode,
    Router,
    routing::{ get, post },
    response::IntoResponse
};
use sysinfo::System;

use crate::{ device::{ Device, DeviceInfo, DevicePublicInfo }, helpers::get_random_sha256, process::{ GithubRepo, Process } };
use dotenvy;
use std::env::var;
use std::{io::Read, sync::Arc};
use tokio::sync::Mutex;
use std::ops::DerefMut;

mod device;
mod helpers;
mod process;

use serde::{ Deserialize, Serialize };

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    dotenvy::dotenv().ok();
    register_node().await.expect("Couldn't register node");

    let this_device = Device {
        hash: helpers::get_random_sha256(),
        device_info: match true {
            true => {
                DeviceInfo::new(None, None).expect("Couldn't get system info")
            },
            false => DeviceInfo::new(None, None).expect("Couldn't get system info"),
        }
    };

    let app = axum::Router::<Arc::<Mutex::<Device>>>::new()
        .route("/", get("Hi"))
        .route("/request_execution", post(request_execution))
        .with_state(Arc::new(Mutex::new(this_device)))
    ;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", var("PORT").expect("No PORT specified in environment variables"))).await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

// TODO - register it on the network as an available node
async fn register_node() -> Result <(), ()>{

    Ok(())
}

// TODO Shift to stream based downloading cause larger files may cause RAM usage spike when
// downloading
async fn request_execution(Json(request_execution_request): Json<RequestExecutionRequest>) -> impl IntoResponse {
    let mutex_device = System::new().unwrap();
    let total_memory_in_megabytes = mutex_device.total_memory() / 1024 / 1024;
    let physical_core_count = System::physical_core_count().expect("couldn't get number of cpu cores on the system");
    let mut mutex_device = Device {
        hash: get_random_sha256(),
        device_info: DeviceInfo {
            device_system: mutex_device,
            physical_core_count: 8,
            allowed_max_core_count: 8,
            total_memory_in_megabytes: 8000,
            allowed_max_memory_in_megabytes: 8000
        }
    };

    match mutex_device.public_info().check_requirements(&request_execution_request.process_requirements) {
        Ok(()) => {
            mutex_device.run_process(
                    request_execution_request.github_repo
            ).await;
            return String::from("process started")
        },
        // Err(e) => { return Ok(Json(RequestExecutionResponse { is_request_accepted: true, reason_for_rejection: Some(e)})) }
        Err(e) => { return String::from("HI") }
    }
    // Ok(Json(RequestExecutionResponse { is_request_accepted: true, reason_for_rejection: None }))
}


#[derive(Deserialize, Serialize)]
pub struct RequestExecutionResponse {
    is_request_accepted: bool,
    reason_for_rejection: Option<process::ProcessError>
}

#[derive(Deserialize, Serialize)]
pub struct RequestExecutionRequest {
    // pub device_public_info: DevicePublicInfo,
    pub github_repo: GithubRepo,
    pub process_requirements: process::ProcessRequirements,
}
