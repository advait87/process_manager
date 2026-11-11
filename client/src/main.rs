#![allow(unused)]

use axum::{
    extract::{ Json, State },
    handler::Handler,
    http::StatusCode,
    Router,
    routing::{ get, post },
    response::IntoResponse
};

use crate::device::{ Device, DeviceInfo, DevicePublicInfo };
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
        device_info: DeviceInfo::new(None, None).expect("Couldn't get system info")
    };

    let app = axum::Router::<Arc::<Mutex::<Device>>>::new()
        .route("/", get(node_info))
        .route("/request_execution", post(request_execution))
        .with_state(Arc::new(Mutex::new(this_device)))
    ;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("HI");
}

// TODO - register it on the network as an available node
async fn register_node() -> Result <(), ()>{

    Ok(())
}

async fn request_execution(Json(body): Json<RequestExecutionRequest>) -> Result<Json<RequestExecutionResponse>, StatusCode> {
    match body.device_public_info.check_requirements(&body.process_requirements) {
        Ok(()) => Ok(Json(RequestExecutionResponse { is_request_accepted: true, reason_for_rejection: None })),
        Err(e) => Ok(Json(RequestExecutionResponse { is_request_accepted: false, reason_for_rejection: Some(e) }))
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
    pub device_public_info: DevicePublicInfo,
    pub process_requirements: process::ProcessRequirements
}
