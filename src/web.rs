use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static UPLOAD: RefCell<Option<(String, String)>> = const { RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
pub fn pick_file() {
    let Some(window) = web_sys::window() else { return; };
    let Some(document) = window.document() else { return; };
    let Ok(element) = document.create_element("input") else { return; };
    let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() else { return; };
    input.set_type("file");
    let _ = input.set_attribute("accept", ".rs,.py,.js,.ts,.json,.md,.html,.css,.c,.cpp,.java,.go,.sql,.sh,.txt");
    let onchange = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Some(target) = event.target() else { return; };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else { return; };
        let Some(files) = input.files() else { return; };
        let Some(file) = files.get(0) else { return; };
        let name = file.name();
        let Ok(reader) = web_sys::FileReader::new() else { return; };
        let reader_for_cb = reader.clone();
        let onload = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Ok(result) = reader_for_cb.result() {
                if let Some(text) = result.as_string() { UPLOAD.with(|slot| *slot.borrow_mut() = Some((name.clone(), text))); }
            }
        }) as Box<dyn FnMut(_)>);
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        let _ = reader.read_as_text(&file);
    }) as Box<dyn FnMut(_)>);
    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    onchange.forget();
    input.click();
}

#[cfg(target_arch = "wasm32")]
pub fn take_uploaded_file() -> Option<(String, String)> { UPLOAD.with(|slot| slot.borrow_mut().take()) }

#[cfg(not(target_arch = "wasm32"))]
pub fn take_uploaded_file() -> Option<(String, String)> { None }
