#![allow(unused)]

use axum::{
    extract::{ Json, State },
    handler::Handler,
    http::StatusCode,
    Router,
    routing::{ get, post },
    response::IntoResponse
};

use crate::{ device::{ Device, DeviceInfo, DevicePublicInfo }, process::Process };
use std::{io::Read, sync::Arc};
use std::sync::Mutex;
use std::ops::DerefMut;

mod device;
mod helpers;
mod process;

use serde::{ Deserialize, Serialize };

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    register_node().await.expect("Couldn't register node");

    let this_device = Device {
        hash: helpers::get_random_sha256(),
        device_info: match true {
            true => {
                println!("sldkjf");
                DeviceInfo::new(None, None).expect("Couldn't get system info")
            },
            false => DeviceInfo::new(None, None).expect("Couldn't get system info"),
        }
    };
    println!("Red");

    let app = axum::Router::<Arc::<Mutex::<Device>>>::new()
        .route("/", get(node_info))
        .route("/request_execution", post(request_execution))
        .with_state(Arc::new(Mutex::new(this_device)))
    ;
    println!("Gray");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Purple");
    axum::serve(listener, app).await.unwrap()
    // println!("Hi");
}

// TODO - register it on the network as an available node
async fn register_node() -> Result <(), ()>{

    Ok(())
}

async fn request_execution(State(arc_mutex_device): State<Arc<Mutex<Device>>>, Json(request_execution_request): Json<RequestExecutionRequest>) -> Result<Json<RequestExecutionResponse>, StatusCode> {
    match arc_mutex_device.lock() {
        Ok(mut mutex_device) => {
            println!("Got arc mutex of the device");
            match mutex_device.public_info().check_requirements(&request_execution_request.process_requirements) {
                Ok(()) => {
                    println!("Running process");
                    match mutex_device.run_process(
                        Process {
                            python_code: request_execution_request.python_code,
                            process_requirements: request_execution_request.process_requirements
                        }
                    ) {
                        Ok(_) => {println!("Process ran")},
                        Err(e) => { return Ok(Json(RequestExecutionResponse { is_request_accepted: true, reason_for_rejection: Some(e)})) }
                    }
                },
                Err(e) => { return Ok(Json(RequestExecutionResponse { is_request_accepted: true, reason_for_rejection: Some(e)})) }
            }
            Ok(Json(RequestExecutionResponse { is_request_accepted: true, reason_for_rejection: None }))
        },
        Err(e) => Ok(Json(RequestExecutionResponse { is_request_accepted: false, reason_for_rejection: Some(process::ProcessError::FailedToCreateFile) }))
    }
}

#[axum::debug_handler]
async fn node_info(State(mut this_device): State<Arc::<Mutex::<Device>>>) -> Result<Json<DevicePublicInfo>, StatusCode> {
    match this_device.lock() {
        Ok(mut this_device) => Ok(Json(this_device.deref_mut().public_info())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Deserialize, Serialize)]
pub struct RequestExecutionResponse {
    is_request_accepted: bool,
    reason_for_rejection: Option<process::ProcessError>
}

#[derive(Deserialize, Serialize)]
pub struct RequestExecutionRequest {
    // pub device_public_info: DevicePublicInfo,
    pub process_requirements: process::ProcessRequirements,
    pub python_code: String
}
