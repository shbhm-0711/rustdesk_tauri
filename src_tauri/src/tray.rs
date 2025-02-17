use crate::ui;

use super::ui_interface::get_option_opt;
#[cfg(target_os = "linux")]
use hbb_common::log::{debug, error, info};
#[cfg(target_os = "linux")]
use libappindicator::AppIndicator;
use tauri::{Builder, Wry};
use tauri::{GlobalShortcutManager, Manager};

#[cfg(target_os = "linux")]
use std::env::temp_dir;
#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "windows")]
use trayicon::{MenuBuilder, TrayIconBuilder};
#[cfg(target_os = "windows")]
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(target_os = "windows")]
#[derive(Clone, Eq, PartialEq, Debug)]
enum Events {
    DoubleClickTrayIcon,
    StopService,
    StartService,
}

// TODO: implement tray managment
pub fn start_tray_tauri(builder: Builder<Wry>) -> Builder<Wry>{
    builder.system_tray(
        tauri::SystemTray::new().with_menu(
            tauri::SystemTrayMenu::new()
                .add_item(tauri::CustomMenuItem::new("remote".to_string(), "Remote"))
                .add_native_item(tauri::SystemTrayMenuItem::Separator)
                .add_item(tauri::CustomMenuItem::new("toggle".to_string(), "Hide"))
                .add_native_item(tauri::SystemTrayMenuItem::Separator)
                .add_item(tauri::CustomMenuItem::new("quit", "Quit")),
        ),
    )
    .on_system_tray_event(move |app, event| match event {
        tauri::SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a left click");
        }
        tauri::SystemTrayEvent::RightClick {
            position: _,
            size: _,
            ..
        } => {
            let window = app.get_window("main").unwrap();
            // update dashboard menu text 
            if window.is_visible().unwrap() {
                app.tray_handle()
                    .get_item("toggle")
                    .set_title("Hide")
                    .unwrap();
            } else {
                app.tray_handle()
                    .get_item("toggle")
                    .set_title("Show")
                    .unwrap();
            }
            println!("system tray received a right click");
        }
        tauri::SystemTrayEvent::DoubleClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a double click");
        }
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "remote" => ui::show_remote_window(app),
                "quit" => {
                    let app = app.clone();
                    std::thread::spawn(move || app.exit(0));
                }
                "toggle" => {
                    let window = app.get_window("main").unwrap();
                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                    }
                }
                _ => {}
            }
        }
        _ => todo!(),
    })
    
}

#[cfg(target_os = "windows")]
pub fn start_tray() {
    let event_loop = EventLoop::<Events>::with_user_event();
    let proxy = event_loop.create_proxy();
    let icon = include_bytes!("../res/tray-icon.ico");
    let mut tray_icon = TrayIconBuilder::new()
        .sender_winit(proxy)
        .icon_from_buffer(icon)
        .tooltip("RustDesk")
        .on_double_click(Events::DoubleClickTrayIcon)
        .build()
        .unwrap();
    let old_state = Arc::new(Mutex::new(0));
    let _sender = crate::ui_interface::SENDER.lock().unwrap();
    event_loop.run(move |event, _, control_flow| {
        if get_option_opt("ipc-closed").is_some() {
            *control_flow = ControlFlow::Exit;
            return;
        } else {
            *control_flow = ControlFlow::Wait;
        }
        let stopped = is_service_stoped();
        let state = if stopped { 2 } else { 1 };
        let old = *old_state.lock().unwrap();
        if state != old {
            hbb_common::log::info!("State changed");
            let mut m = MenuBuilder::new();
            if state == 2 {
                m = m.item(
                    &crate::client::translate("Start Service".to_owned()),
                    Events::StartService,
                );
            } else {
                m = m.item(
                    &crate::client::translate("Stop service".to_owned()),
                    Events::StopService,
                );
            }
            tray_icon.set_menu(&m).ok();
            *old_state.lock().unwrap() = state;
        }

        match event {
            Event::UserEvent(e) => match e {
                Events::DoubleClickTrayIcon => {
                    crate::run_me(Vec::<&str>::new()).ok();
                }
                Events::StopService => {
                    crate::ipc::set_option("stop-service", "Y");
                }
                Events::StartService => {
                    crate::ipc::set_option("stop-service", "");
                }
            },
            _ => (),
        }
    });
}

/// Start a tray icon in Linux
///
/// [Block]
/// This function will block current execution, show the tray icon and handle events.
#[cfg(target_os = "linux")]
pub fn start_tray() {
    use gtk::traits::{GtkMenuItemExt, MenuShellExt, WidgetExt};

    info!("configuring tray");
    // init gtk context
    if let Err(err) = gtk::init() {
        error!("Error when starting the tray: {}", err);
        return;
    }
    if let Some(mut appindicator) = get_default_app_indicator() {
        let mut menu = gtk::Menu::new();
        let stoped = is_service_stoped();
        // start/stop service
        let label = if stoped {
            crate::client::translate("Start Service".to_owned())
        } else {
            crate::client::translate("Stop service".to_owned())
        };
        let menu_item_service = gtk::MenuItem::with_label(label.as_str());
        menu_item_service.connect_activate(move |item| {
            let _lock = crate::ui_interface::SENDER.lock().unwrap();
            update_tray_service_item(item);
        });
        menu.append(&menu_item_service);
        // show tray item
        menu.show_all();
        appindicator.set_menu(&mut menu);
        // start event loop
        info!("Setting tray event loop");
        gtk::main();
    } else {
        error!("Tray process exit now");
    }
}

#[cfg(target_os = "linux")]
fn update_tray_service_item(item: &gtk::MenuItem) {
    use gtk::traits::GtkMenuItemExt;

    if is_service_stoped() {
        debug!("Now try to start service");
        item.set_label(&crate::client::translate("Stop service".to_owned()));
        crate::ipc::set_option("stop-service", "");
    } else {
        debug!("Now try to stop service");
        item.set_label(&crate::client::translate("Start Service".to_owned()));
        crate::ipc::set_option("stop-service", "Y");
    }
}

#[cfg(target_os = "linux")]
fn get_default_app_indicator() -> Option<AppIndicator> {
    use libappindicator::AppIndicatorStatus;
    use std::io::Write;

    let icon = include_bytes!("../res/icon.png");
    // appindicator does not support icon buffer, so we write it to tmp folder
    let mut icon_path = temp_dir();
    icon_path.push("RustDesk");
    icon_path.push("rustdesk.png");
    match std::fs::File::create(icon_path.clone()) {
        Ok(mut f) => {
            f.write_all(icon).unwrap();
        }
        Err(err) => {
            error!("Error when writing icon to {:?}: {}", icon_path, err);
            return None;
        }
    }
    debug!("write temp icon complete");
    let mut appindicator = AppIndicator::new("RustDesk", icon_path.to_str().unwrap_or("rustdesk"));
    appindicator.set_label("RustDesk", "A remote control software.");
    appindicator.set_status(AppIndicatorStatus::Active);
    Some(appindicator)
}

/// Check if service is stoped.
/// Return [`true`] if service is stoped, [`false`] otherwise.
#[inline]
fn is_service_stoped() -> bool {
    if let Some(v) = get_option_opt("stop-service") {
        v == "Y"
    } else {
        false
    }
}
