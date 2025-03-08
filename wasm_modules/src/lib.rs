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

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn greeting_code() -> i32 {
    42
}

#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// no worky cause pointer to js!!
#[no_mangle]
pub extern "C" fn greet() -> *mut u8 {
    b"Hello from webassembly!".as_ptr() as *mut u8
}
