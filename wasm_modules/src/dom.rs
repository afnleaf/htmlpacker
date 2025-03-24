use wasm_bindgen::prelude::*;

// function to create and set up the canvas element
// inside the dom
pub fn create_canvas() -> Result<(), JsValue> {
    // window and document DOM
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    // create canvas element
    let canvas = document.create_element("canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    
    // window dimensions
    let width = window.inner_width()?.as_f64().unwrap() as u32;
    let height = window.inner_height()?.as_f64().unwrap() as u32;
    
    // Set canvas dimensions and ID
    canvas.set_width(width);
    canvas.set_height(height);
    canvas.set_id("canvas");
    
    // should this be in a css file?
    // no we don't need css besides this
    canvas.style().set_property("display", "block")?;
    canvas.style().set_property("margin", "0")?;
    canvas.style().set_property("padding", "0")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    
    // append canvas to DOM
    document.body().unwrap().append_child(&canvas)?;
    
    // body styles full screen
    document.body().unwrap().style().set_property("margin", "0")?;
    document.body().unwrap().style().set_property("padding", "0")?;
    document.body().unwrap().style().set_property("overflow", "hidden")?;
    document.body().unwrap().style().set_property("width", "100vw")?;
    document.body().unwrap().style().set_property("height", "100vh")?;
    
    // console log allows for debugging in browser
    web_sys::console::log_1(&"Canvas created successfully".into());
    moyai()?;
    moyai()?;
    
    Ok(())
}

fn moyai() -> Result<(), JsValue> {
    web_sys::console::log_2(
        &JsValue::from_str("%c "),
        &JsValue::from_str("font-size:250px;background:url(\"data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'><text y='0.95em' font-size='8'>👽</text></svg>\")"));
        //&JsValue::from_str("font-size:250px;background:url(\"data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg'> viewbox='0 0 10 10'><text y='0.95em' font-size='8'>🗿</text></svg>\")"));
    Ok(())
}
