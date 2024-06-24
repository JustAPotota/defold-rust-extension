//! Raw bindings generated by rust-bindgen. Only use this if you know what you're doing!

#![allow(warnings)]

#[cfg(target_os = "linux")]
mod bindings {
    include!("bindings-x86_64-unknown-linux-gnu.rs");
}

#[cfg(target_os = "windows")]
mod bindings {
    include!("bindings-x86_64-pc-windows-msv.rs");
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod bindings {
    include!("bindings-x86_64-apple-darwin.rs");
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod bindings {
    include!("bindings-aarch64-apple-darwin.rs");
}

pub use bindings::root::*;
