use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Mutex;
use std::{collections::HashMap, fs};
use wake_on_lan::MagicPacket;
use warp::Filter;

const DEVICE_FILE: &str = "/app/config/devices.json";

lazy_static! {
    static ref DEVICES: Mutex<HashMap<String, String>> = Mutex::new(load_devices());
}

#[derive(Deserialize)]
struct WakeRequest {
    mac: Option<String>,
    ip: Option<String>,
    token: Option<String>,
    device_name: Option<String>,
}

#[derive(Deserialize)]
struct DeviceRequest {
    name: String,
    mac: String,
    token: Option<String>,
}

#[derive(Deserialize)]
struct DeleteRequest {
    name: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct Device {
    name: String,
    mac: String,
}

fn load_devices() -> HashMap<String, String> {
    match fs::read_to_string(DEVICE_FILE) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_devices(map: &HashMap<String, String>) {
    let _ = fs::write(DEVICE_FILE, serde_json::to_string_pretty(map).unwrap());
}

fn mac_string_to_bytes(s: &str) -> Result<[u8; 6], ()> {
    let parts: Vec<&str> = s.split(|c| c == ':' || c == '-').collect();
    if parts.len() != 6 {
        return Err(());
    }
    let mut bytes = [0u8; 6];
    for i in 0..6 {
        bytes[i] = u8::from_str_radix(parts[i], 16).map_err(|_| ())?;
    }
    Ok(bytes)
}

fn check_token(api_token: &Option<String>, req_token: &Option<String>) -> bool {
    match api_token {
        Some(t) => req_token.as_deref() == Some(t),
        None => true,
    }
}

#[tokio::main]
async fn main() {
    let api_token = std::env::var("API_TOKEN").ok();

    // GET /wake
    let api_token_filter = api_token.clone();
    let wake = warp::path("wake")
        .and(warp::get())
        .and(warp::query::<WakeRequest>())
        .map(move |req: WakeRequest| {
            if !check_token(&api_token_filter, &req.token) {
                return warp::reply::with_status(
                    "Unauthorized",
                    warp::http::StatusCode::UNAUTHORIZED,
                );
            }

            let mac = if let Some(name) = &req.device_name {
                DEVICES.lock().unwrap().get(name).cloned()
            } else {
                req.mac.clone()
            };

            let mac = match mac {
                Some(m) => m,
                None => {
                    return warp::reply::with_status(
                        "MAC not found",
                        warp::http::StatusCode::BAD_REQUEST,
                    )
                }
            };

            let mac_bytes = match mac_string_to_bytes(&mac) {
                Ok(b) => b,
                Err(_) => {
                    return warp::reply::with_status(
                        "Invalid MAC",
                        warp::http::StatusCode::BAD_REQUEST,
                    )
                }
            };

            let ip_addr = req
                .ip
                .unwrap_or_else(|| "255.255.255.255".to_string())
                .parse::<Ipv4Addr>();
            let ip_addr = match ip_addr {
                Ok(ip) => ip,
                Err(_) => {
                    return warp::reply::with_status(
                        "Invalid IP",
                        warp::http::StatusCode::BAD_REQUEST,
                    )
                }
            };

            let to_addr = SocketAddr::V4(SocketAddrV4::new(ip_addr, 9));
            let from_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
            let packet = MagicPacket::new(&mac_bytes);
            match packet.send_to(&to_addr, &from_addr) {
                Ok(_) => warp::reply::with_status("Sent WOL packet", warp::http::StatusCode::OK),
                Err(_) => warp::reply::with_status(
                    "Failed to send WOL",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ),
            }
        });

    // GET /devices
    let devices_list = warp::path("devices").and(warp::get()).map(|| {
        let map = DEVICES.lock().unwrap();
        let list: Vec<Device> = map
            .iter()
            .map(|(k, v)| Device {
                name: k.clone(),
                mac: v.clone(),
            })
            .collect();
        warp::reply::json(&list)
    });

    // POST /devices
    let api_token_filter2 = api_token.clone();
    let devices_add = warp::path("devices")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: DeviceRequest| {
            if !check_token(&api_token_filter2, &req.token) {
                return warp::reply::with_status(
                    "Unauthorized",
                    warp::http::StatusCode::UNAUTHORIZED,
                );
            }
            let mut map = DEVICES.lock().unwrap();
            map.insert(req.name.clone(), req.mac.clone());
            save_devices(&map);
            warp::reply::with_status("Device added", warp::http::StatusCode::OK)
        });

    // DELETE /devices
    let api_token_filter3 = api_token.clone();
    let devices_remove = warp::path("devices")
        .and(warp::delete())
        .and(warp::query::<DeleteRequest>())
        .map(move |req: DeleteRequest| {
            if !check_token(&api_token_filter3, &req.token) {
                return warp::reply::with_status(
                    "Unauthorized",
                    warp::http::StatusCode::UNAUTHORIZED,
                );
            }
            let mut map = DEVICES.lock().unwrap();
            map.remove(&req.name);
            save_devices(&map);
            warp::reply::with_status("Device removed", warp::http::StatusCode::OK)
        });

    // Static files from /web
    let static_files = warp::path("web").and(warp::fs::dir("/app/web"));

    println!("Server running on http://0.0.0.0:8080");
    let routes = wake
        .or(devices_list)
        .or(devices_add)
        .or(devices_remove)
        .or(static_files);
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
