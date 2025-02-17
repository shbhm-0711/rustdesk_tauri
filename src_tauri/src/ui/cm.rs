#[cfg(target_os = "linux")]
use crate::ipc::start_pa;
use crate::ui_cm_interface::{start_ipc, ConnectionManager, InvokeUiCM};
use tauri::Manager;

use hbb_common::{allow_err, log};
use sciter::{make_args, Element, Value, HELEMENT};
use std::sync::Mutex;
use std::{ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};

use super::get_app_handle;
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct TauriHandler;

impl InvokeUiCM for TauriHandler {
    fn add_connection(&self, client: &crate::ui_cm_interface::Client) {
        
        let app_handle: Option<tauri::AppHandle> = get_app_handle();
        match app_handle {
            Some(app) => {
                app.emit_all("addConnection", client).unwrap();
            }
            None => {
                log::info!("Waiting to get app handle for macro to execute...");
            }
        }
    }

    fn remove_connection(&self, id: i32, close: bool) {
        // self.call("removeConnection", &make_args!(id, close));
        if crate::ui_cm_interface::get_clients_length().eq(&0) {
            crate::platform::quit_gui();
        }
    }

    fn new_message(&self, id: i32, text: String) {
        // self.call("newMessage", &make_args!(id, text));
    }

    fn change_theme(&self, _dark: String) {
        // TODO
    }

    fn change_language(&self) {
        // TODO
    }

    fn show_elevation(&self, show: bool) {
        // self.call("showElevation", &make_args!(show));
    }
}

impl TauriHandler {
    #[inline]
    fn call(&self, app: &tauri::AppHandle, func: &str, args: &[String]) {
        // if let Some(e) = self.element.lock().unwrap().as_ref() {
        //     allow_err!(e.call_method(func, &super::value_crash_workaround(args)[..]));
        // }
        app.emit_all(func, args).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TauriConnectionManager(ConnectionManager<TauriHandler>);

impl Deref for TauriConnectionManager {
    type Target = ConnectionManager<TauriHandler>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TauriConnectionManager {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        std::thread::spawn(start_pa);
        let cm = ConnectionManager {
            ui_handler: TauriHandler::default(),
        };
        let cloned = cm.clone();
        std::thread::spawn(move || start_ipc(cloned));
        TauriConnectionManager(cm)
    }

    fn get_icon(&mut self) -> String {
        crate::get_icon()
    }

    fn check_click_time(&mut self, id: i32) {
        crate::ui_cm_interface::check_click_time(id);
    }

    fn get_click_time(&self) -> f64 {
        crate::ui_cm_interface::get_click_time() as _
    }

    fn switch_permission(&self, id: i32, name: String, enabled: bool) {
        crate::ui_cm_interface::switch_permission(id, name, enabled);
    }

    fn close(&self, id: i32) {
        crate::ui_cm_interface::close(id);
    }

    fn remove_disconnected_connection(&self, id: i32) {
        crate::ui_cm_interface::remove(id);
    }

    fn quit(&self) {
        crate::platform::quit_gui();
    }

    fn authorize(&self, id: i32) {
        crate::ui_cm_interface::authorize(id);
    }

    fn send_msg(&self, id: i32, text: String) {
        crate::ui_cm_interface::send_chat(id, text);
    }

    fn t(&self, name: String) -> String {
        crate::client::translate(name)
    }

    fn can_elevate(&self) -> bool {
        crate::ui_cm_interface::can_elevate()
    }

    fn elevate_portable(&self, id: i32) {
        crate::ui_cm_interface::elevate_portable(id);
    }
}

impl sciter::EventHandler for TauriConnectionManager {
    // fn attached(&mut self, root: HELEMENT) {
    //     // TODO:
    //     // *self.ui_handler.element.lock().unwrap() = Some(Element::from(root));
    // }

    sciter::dispatch_script_call! {
        fn t(String);
        fn check_click_time(i32);
        fn get_click_time();
        fn get_icon();
        fn close(i32);
        fn remove_disconnected_connection(i32);
        fn quit();
        fn authorize(i32);
        fn switch_permission(i32, String, bool);
        fn send_msg(i32, String);
        fn can_elevate();
        fn elevate_portable(i32);
    }
}
