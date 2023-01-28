use dap::{DapService, EventPoster};
use serde::{Deserialize, Serialize};

mod dap;

struct UserData {
    event_poster: EventPoster,
}

impl UserData {
    fn initialize(
        &mut self,
        initialize_requst_args: InitializeRequestArguments,
    ) -> InitializeResponse {
        // TODO: record initialize_requst_args somewhere...

        // TODO: start interpreter

        self.event_poster.queue_event(&InitializeEvent::new());

        InitializeResponse {
            body: Some(Capabilities {
                supports_single_thread_execution_requests: Some(true),
            }),
        }
    }
}

#[derive(Deserialize)]
struct InitializeRequestArguments {
    #[serde(rename(deserialize = "adapterID"))]
    adapter_id: String,
}

// TODO: include content-length and seq, type ...
#[derive(Serialize)]
struct InitializeResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Capabilities>,
}

// TODO: include content-length
#[derive(Serialize)]
struct InitializeEvent {
    #[serde(rename(serialize = "type"))]
    event_type: String,
    event: String,
}

impl InitializeEvent {
    pub fn new() -> Self {
        InitializeEvent {
            event_type: "event".to_string(),
            event: "initialized".to_string(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Capabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    supports_single_thread_execution_requests: Option<bool>,
}

fn main() {
    let mut dap_service = DapService::new_with_poster(|event_poster| UserData { event_poster })
        .register("initialize".to_string(), Box::new(UserData::initialize))
        .build();
    dap_service.start();
}

#[test]
fn test_initialization_request() {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::{thread, time};

    let mut child = Command::new("cargo")
        .args(["run"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed during cargo run");

    let child_stdin = child.stdin.as_mut().unwrap();
    let child_stdout = child.stdout.as_mut().unwrap();
    let initialization_request = "Content-Length: 128\r\n\r\n{\r\n    \"seq\": 153,\r\n    \"type\": \"request\",\r\n    \"command\": \"initialize\",\r\n    \"arguments\": {\r\n        \"adapterID\": \"a\"\r\n    }\r\n}\r\n";
    child_stdin
        .write_all(initialization_request.as_bytes())
        .unwrap();
    // Close stdin to finish and avoid indefinite blocking
    drop(child_stdin);
    thread::sleep(time::Duration::from_secs(5));

    let mut read_buf: [u8; 200] = [0; 200];
    child_stdout.read(&mut read_buf).unwrap();
    child.kill().unwrap();

    let actual = String::from_utf8(read_buf.to_vec()).unwrap();
    assert!(actual.contains("{\"body\":{\"supportsSingleThreadExecutionRequests\":true}}\r\n{\"type\":\"event\",\"event\":\"initialized\"}\r\n"));
}
