//! # DBus interface proxy for: `org.freedesktop.UPower.KbdBacklight`
//!
//! This code was generated by `zbus-xmlgen` `2.0.1` from DBus introspection data.
//! Source: `Interface '/org/freedesktop/UPower/KbdBacklight' from service 'org.freedesktop.UPower' on system bus`.
use cctk::{
    sctk::{output::OutputInfo, reexports::calloop},
    toplevel_info::ToplevelInfo,
    wayland_client::protocol::wl_output::WlOutput,
    wayland_protocols::ext::{
        foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
        workspace::v1::client::ext_workspace_handle_v1::ExtWorkspaceHandleV1,
    },
};
use cosmic::{
    iced::{self, stream, Subscription},
    iced_core::image::Bytes,
};
use image::EncodableLayout;

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver},
    SinkExt, StreamExt,
};
use once_cell::sync::Lazy;
use std::fmt::Debug;
use tokio::sync::Mutex;

use crate::wayland_handler::wayland_handler;

pub static WAYLAND_RX: Lazy<Mutex<Option<UnboundedReceiver<WaylandUpdate>>>> =
    Lazy::new(|| Mutex::new(None));

pub fn wayland_subscription() -> iced::Subscription<WaylandUpdate> {
    Subscription::run_with_id(
        std::any::TypeId::of::<WaylandUpdate>(),
        stream::channel(50, move |mut output| async move {
            let mut state = State::Waiting;

            loop {
                state = start_listening(state, &mut output).await;
            }
        }),
    )
}

pub enum State {
    Waiting,
    Finished,
}

#[derive(Debug, Clone)]
pub struct WaylandImage {
    pub img: Bytes,
    pub width: u32,
    pub height: u32,
}

impl WaylandImage {
    pub fn new(img: image::RgbaImage) -> Self {
        Self {
            // TODO avoid copy?
            img: Bytes::copy_from_slice(img.as_bytes()),
            width: img.width(),
            height: img.height(),
        }
    }
}

impl AsRef<[u8]> for WaylandImage {
    fn as_ref(&self) -> &[u8] {
        self.img.as_bytes()
    }
}

async fn start_listening(
    state: State,
    output: &mut futures::channel::mpsc::Sender<WaylandUpdate>,
) -> State {
    match state {
        State::Waiting => {
            let mut guard = WAYLAND_RX.lock().await;
            let rx = {
                if guard.is_none() {
                    let (calloop_tx, calloop_rx) = calloop::channel::channel();
                    let (toplevel_tx, toplevel_rx) = unbounded();
                    let _ = std::thread::spawn(move || {
                        wayland_handler(toplevel_tx, calloop_rx);
                    });
                    *guard = Some(toplevel_rx);
                    _ = output.send(WaylandUpdate::Init(calloop_tx)).await;
                }
                guard.as_mut().unwrap()
            };
            match rx.next().await {
                Some(u) => {
                    _ = output.send(u).await;
                    State::Waiting
                }
                None => {
                    _ = output.send(WaylandUpdate::Finished).await;
                    tracing::error!("Wayland handler thread died");
                    State::Finished
                }
            }
        }
        State::Finished => iced::futures::future::pending().await,
    }
}

#[derive(Clone, Debug)]
pub enum WaylandUpdate {
    Init(calloop::channel::Sender<WaylandRequest>),
    Finished,
    Toplevel(ToplevelUpdate),
    Workspace(Vec<ExtWorkspaceHandleV1>),
    Output(OutputUpdate),
    ActivationToken {
        token: Option<String>,
        app_id: Option<String>,
        exec: String,
        gpu_idx: Option<usize>,
        terminal: bool,
    },
    Image(ExtForeignToplevelHandleV1, WaylandImage),
}

#[derive(Clone, Debug)]
pub enum ToplevelUpdate {
    Add(ToplevelInfo),
    Update(ToplevelInfo),
    Remove(ExtForeignToplevelHandleV1),
}

#[derive(Clone, Debug)]
pub enum OutputUpdate {
    Add(WlOutput, OutputInfo),
    Update(WlOutput, OutputInfo),
    Remove(WlOutput),
}

#[derive(Clone, Debug)]
pub enum WaylandRequest {
    Toplevel(ToplevelRequest),
    TokenRequest {
        app_id: String,
        exec: String,
        gpu_idx: Option<usize>,
        terminal: bool,
    },
    Screencopy(ExtForeignToplevelHandleV1),
}

#[derive(Debug, Clone)]
pub enum ToplevelRequest {
    Activate(ExtForeignToplevelHandleV1),
    Minimize(ExtForeignToplevelHandleV1),
    Quit(ExtForeignToplevelHandleV1),
}
