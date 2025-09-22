mod layout;

use layout::Layout;
use log::LevelFilter;
use std::env;
use std::io;
use std::process::exit;
use systemd_journal_logger::JournalLog;

use tokio::io::AsyncBufReadExt;
use tokio::io::BufStream;
use tokio::net::UnixStream;

use std::collections::hash_map::HashMap;

static ACTIVE_LAYOUT_KEY: &str = "activelayout>>";
static ACTIVE_WINDOW_KEY: &str = "activewindowv2>>";
static CLOSE_WINDOW_KEY: &str = "closewindow>>";
static OPEN_WOFI_KEY: &str = "openlayer>>wofi";
static DEFAULT_LAYOUT: &str = "English (US)";
static CURRENT_KEYBOARD: &str = "current";

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    initialize_systemd_logger();
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();
    let sock2_path = format!(
        "{}/hypr/{}/.socket2.sock",
        xdg_runtime_dir, hyprland_instance_signature
    );
    let sock_path = format!(
        "{}/hypr/{}/.socket.sock",
        xdg_runtime_dir, hyprland_instance_signature
    );
    log::info!("sock2_path is {}", sock2_path);
    log::info!("sock_path is {}", sock2_path);
    let stream = UnixStream::connect(sock2_path).await?;
    let mut events_socket_stream = BufStream::new(stream);
    let mut buf: Vec<u8> = vec![];
    let mut current_active_window_id: Option<String> = None;
    let mut windows_layouts: HashMap<String, Layout> = HashMap::new();
    loop {
        let bytes_read = events_socket_stream.read_until(b'\n', &mut buf).await?;
        if bytes_read == 0 {
            log::info!("reached eof, exiting");
            exit(0);
        }
        let event: String = String::from_utf8(buf.clone()).unwrap();
        buf.clear();
        if event.starts_with(ACTIVE_WINDOW_KEY) {
            on_active_window_change(
                event,
                &mut windows_layouts,
                &mut current_active_window_id,
                &sock_path,
            )
            .await?;
        } else if event.starts_with(ACTIVE_LAYOUT_KEY) {
            on_active_layout_change(event, &current_active_window_id, &mut windows_layouts).await?;
        } else if event.starts_with(CLOSE_WINDOW_KEY) {
            on_window_close(event, &mut windows_layouts).await?;
        } else if event.starts_with(OPEN_WOFI_KEY) {
            on_wofi_open(&sock_path).await?;
        }
    }
}

async fn switch_layout(sock_path: &str, layout: &Layout) -> std::io::Result<()> {
    if let Some(code) = layout.code() {
        let cmd = format!("switchxkblayout {} {}\n", CURRENT_KEYBOARD, code);
        log::info!("cmd: {}", cmd);
        loop {
            let hyprctl_sock = UnixStream::connect(sock_path).await?;
            hyprctl_sock.writable().await?;
            log::info!("hyprctl_sock is writable");
            match hyprctl_sock.try_write(cmd.as_bytes()) {
                Ok(_n) => {
                    log::info!("layout set to {}", code);
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
    Ok(())
}

fn initialize_systemd_logger() {
    JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(LevelFilter::Error);
    log::info!("systemd logger initialized");
}

async fn on_active_window_change(
    event: String,
    windows_layouts: &mut HashMap<String, Layout>,
    current_active_window_id: &mut Option<String>,
    sock_path: &str,
) -> std::io::Result<()> {
    let (_, id) = event.trim().split_at(ACTIVE_WINDOW_KEY.len());
    log::info!("active window changed to id {}", id);
    log::info!("layouts map size: {}", windows_layouts.len());
    *current_active_window_id = Some(id.to_string());
    if let Some(layout) = windows_layouts.get(id) {
        log::info!(
            "{} is needed to be set for window id {}",
            layout.get_layout(),
            id
        );
        switch_layout(&sock_path, &layout).await?;
    } else {
        log::info!("setting default layout for window_id {}", id);
        let layout = Layout::new(DEFAULT_LAYOUT.to_string());
        switch_layout(&sock_path, &layout).await?;
        windows_layouts.insert(id.to_string(), layout);
    }
    Ok(())
}

async fn on_active_layout_change(
    event: String,
    current_active_window_id: &Option<String>,
    windows_layouts: &mut HashMap<String, Layout>,
) -> std::io::Result<()> {
    let (_, params) = event.split_at(ACTIVE_LAYOUT_KEY.len());
    let (_, layout) = params.trim().split_at(params.find(',').unwrap());
    let layout = &layout[1..];
    log::info!(
        "keyboard layout changed, `{}`, current window id is `{}`",
        layout,
        current_active_window_id
            .clone()
            .unwrap_or("NONE".to_string())
    );
    if let Some(ref window_id) = current_active_window_id {
        windows_layouts.insert(window_id.to_string(), Layout::new(layout.to_string()));
    }
    Ok(())
}

async fn on_window_close(
    event: String,
    windows_layouts: &mut HashMap<String, Layout>,
) -> std::io::Result<()> {
    let (_, closed_id) = event.trim().split_at(CLOSE_WINDOW_KEY.len());
    log::info!(
        "Window with id {} is closed - will delete it from layouts map",
        closed_id
    );
    windows_layouts.remove(closed_id);
    Ok(())
}

async fn on_wofi_open(sock_path: &str) -> std::io::Result<()> {
    log::info!("Wofi opened - setting layout to default");
    switch_layout(&sock_path, &Layout::new(DEFAULT_LAYOUT.to_string())).await
}
