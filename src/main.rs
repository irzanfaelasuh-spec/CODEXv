#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;
    let document = web_sys::window().expect("window").document().expect("document");
    let canvas = document.get_element_by_id("the_canvas_id").expect("canvas").dyn_into::<web_sys::HtmlCanvasElement>().expect("canvas element");
    wasm_bindgen_futures::spawn_local(async move {
        let web_options = eframe::WebOptions::default();
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(vyron_ide::app::VyronApp::new(cc)))),
            )
            .await;
        if let Err(err) = start_result { log::error!("VYRON failed to start: {err:?}"); }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("VYRON IDE").with_inner_size([1440.0, 900.0]).with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("VYRON IDE", options, Box::new(|cc| Ok(Box::new(vyron_ide::app::VyronApp::new(cc)))))
}
