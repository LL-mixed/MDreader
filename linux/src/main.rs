// MDreader — native Linux Markdown reader (GTK4 + WebKitGTK6).
// LM1: register embedded GResource + `mdreader://` scheme, open a window rendering shared/render.

mod render;

use gtk::prelude::*;
use gtk::Application;

const APP_ID: &str = "com.mdreader.MDreader";

fn main() {
    gio::resources_register_include!("render.gresource").expect("failed to register gresource");
    render::webview::register_scheme();

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        let win = gtk::ApplicationWindow::builder()
            .application(app)
            .title("MDreader")
            .default_width(1000)
            .default_height(640)
            .build();
        let wv = render::webview::new_webview();
        win.set_child(Some(&wv));
        win.present();
    });

    app.run();
}
