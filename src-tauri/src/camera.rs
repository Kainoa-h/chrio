use std::io::Cursor;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::{extract::State as AxumState, routing::get, Router};
use image::ImageFormat;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
    Resolution,
};
use nokhwa::Camera;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::watch;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CameraDevice {
    pub index: u32,
    pub name: String,
}

pub struct CameraState {
    pub frame_tx: watch::Sender<Arc<Vec<u8>>>,
    pub frame_rx: watch::Receiver<Arc<Vec<u8>>>,
    pub camera_handle: std::sync::Mutex<Option<CameraHandle>>,
    pub stream_port: u16,
}

pub struct CameraHandle {
    stop_tx: std::sync::mpsc::Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for CameraHandle {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn enumerate_cameras() -> Result<Vec<CameraDevice>, String> {
    let devices = nokhwa::query(ApiBackend::Auto).map_err(|e| e.to_string())?;
    Ok(devices
        .into_iter()
        .enumerate()
        .map(|(i, info)| CameraDevice {
            index: i as u32,
            name: info.human_name().to_string(),
        })
        .collect())
}

pub fn start_capture(
    device_index: u32,
    frame_tx: watch::Sender<Arc<Vec<u8>>>,
) -> Result<CameraHandle, String> {
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();

    let join_handle = std::thread::spawn(move || {
        // Request MJPEG at 720p/30fps — if the camera supports MJPEG,
        // frames arrive as JPEG and skip encoding entirely.
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
            CameraFormat::new(Resolution::new(1280, 720), FrameFormat::MJPEG, 30),
        ));
        let mut camera = match Camera::new(CameraIndex::Index(device_index), format) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to open camera: {}", e);
                return;
            }
        };

        // Log what format we actually got
        eprintln!(
            "Camera opened: {:?} (requested MJPEG 1280x720@30)",
            camera.camera_format()
        );

        if let Err(e) = camera.open_stream() {
            eprintln!("Failed to open camera stream: {}", e);
            return;
        }

        let mut first_frame = true;
        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            match camera.frame() {
                Ok(frame) => {
                    if first_frame {
                        eprintln!(
                            "First frame: format={:?}, resolution={}x{}",
                            frame.source_frame_format(),
                            frame.resolution().width(),
                            frame.resolution().height()
                        );
                        first_frame = false;
                    }
                    let jpeg_bytes = if frame.source_frame_format() == FrameFormat::MJPEG {
                        Some(frame.buffer().to_vec())
                    } else {
                        frame.decode_image::<RgbFormat>().ok().and_then(|rgb| {
                            let mut buf = Cursor::new(Vec::new());
                            rgb.write_to(&mut buf, ImageFormat::Jpeg).ok()?;
                            Some(buf.into_inner())
                        })
                    };
                    if let Some(bytes) = jpeg_bytes {
                        let _ = frame_tx.send(Arc::new(bytes));
                    }
                }
                Err(e) => {
                    eprintln!("Frame capture error: {}", e);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        let _ = camera.stop_stream();
    });

    Ok(CameraHandle {
        stop_tx,
        join_handle: Some(join_handle),
    })
}

pub fn stop_capture(state: &CameraState) {
    let mut handle = state.camera_handle.lock().unwrap();
    *handle = None;
    let _ = state.frame_tx.send(Arc::new(Vec::new()));
}

pub fn snap_frame(state: &CameraState) -> Result<Vec<u8>, String> {
    let frame = state.frame_rx.borrow().clone();
    if frame.is_empty() {
        return Err("No frame available — is the camera running?".to_string());
    }
    Ok((*frame).clone())
}

// WebSocket streaming server

#[derive(Clone)]
struct WsState {
    frame_rx: watch::Receiver<Arc<Vec<u8>>>,
}

async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<WsState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| ws_stream_handler(socket, state))
}

async fn ws_stream_handler(mut socket: WebSocket, state: WsState) {
    let mut rx = state.frame_rx.clone();
    loop {
        // Wait for a new frame from the camera thread
        if rx.changed().await.is_err() {
            break; // Sender dropped
        }
        let frame = rx.borrow_and_update().clone();
        if frame.is_empty() {
            continue;
        }
        // Send raw JPEG bytes as a binary WebSocket message
        if socket
            .send(Message::Binary(frame.to_vec().into()))
            .await
            .is_err()
        {
            break; // Client disconnected
        }
    }
}

pub async fn start_stream_server(frame_rx: watch::Receiver<Arc<Vec<u8>>>) -> u16 {
    let state = WsState { frame_rx };
    let app = Router::new()
        .route("/stream", get(ws_upgrade_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind stream server");
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    port
}
