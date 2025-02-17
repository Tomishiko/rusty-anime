use std::{
    io::{self, Read, Write},
    vec,
};

use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![read_streamed_data])
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window.hide().unwrap();

                io::stdout().write_all(&[0u8]).expect("error");
                io::stdout().flush().expect("Unable to write stdout");
                window
                    .emit("reload_event", true)
                    .expect("Unable to emite event to js frontend");
            }

            _ => {}
        })
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app
                    .get_webview_window("main")
                    .expect("Unable to get main window");
                window.open_devtools();
            }

            Ok(())
        })
        .build(context)
        .unwrap();
    app.run(|_, _| {});
}
#[tauri::command]
fn read_streamed_data(window: tauri::Window) -> String {
    let mut stream_in = io::stdin();
    let mut buf = [0u8; 4];
    stream_in.read(&mut buf).expect("Unable to read stdin");
    let content_length = u32::from_le_bytes(buf);
    let mut content = vec![0u8; content_length as usize];
    let bytes_read = stream_in.read(&mut content).expect("Unable to read stdin");
    let content = unsafe { String::from_utf8_unchecked(content) };
    window.show().expect("unable to show window");
    window.set_focus().expect("Unable to focus window");

    return content;
}
