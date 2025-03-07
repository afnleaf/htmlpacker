//use wasm_bindgen::prelude::*;
//
//#[wasm_bindgen]
//pub fn greet() -> String {
//    "Hello from webassembly!".to_string()
//}
//
//
//
/*
#[no_mangle]
pub extern "C" fn greet() -> *mut u8 {
    b"Hello from webassembly!".to_ptr() as *mut u8
}
*/
// Simple addition function
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Return a greeting code
#[no_mangle]
pub extern "C" fn greeting_code() -> i32 {
    42
}

// Multiply two numbers
#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
