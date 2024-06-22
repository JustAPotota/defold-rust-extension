//! Configuration file access functions. The configuration file is the compiled version of game.project.

use dmsdk_ffi::dmConfigFile;
use libc::c_void;
use std::ffi::{c_char, CStr, CString};

#[doc(hidden)]
pub type RawConfigFile = dmConfigFile::HConfig;

/// Handle of the project's configuration file.
#[derive(Debug, Clone, Copy)]
pub struct ConfigFile {
    ptr: dmConfigFile::HConfig,
}

impl From<dmConfigFile::HConfig> for ConfigFile {
    fn from(ptr: dmConfigFile::HConfig) -> Self {
        Self { ptr }
    }
}

impl From<ConfigFile> for dmConfigFile::HConfig {
    fn from(config: ConfigFile) -> Self {
        config.ptr
    }
}

/// Gets the corresponding config value as a String.
///
/// `default_value` will be returned if the key isn't found.
///
/// # Examples
/// ```
/// # const LOG_DOMAIN: &str = "DOCTEST";
/// use dmsdk::*;
///
/// fn app_init(params: dmextension::AppParams) -> dmextension::Result {
///     let title = dmconfigfile::get_string(params.config, "project.title", "Untitled");
///     dmlog::info!("Project title is: {title}");
///
///     dmextension::Result::Ok
/// }
/// ```
pub fn get_string(config: ConfigFile, key: &str, default_value: &str) -> String {
    let key = CString::new(key).unwrap();
    let default_value = CString::new(default_value).unwrap();

    let ptr =
        unsafe { dmConfigFile::GetString(config.into(), key.as_ptr(), default_value.as_ptr()) };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    String::from_utf8_lossy(cstr.to_bytes()).into_owned()
}

/// Gets the corresponding config value as an i32.
///
/// `default_value` will be returned if the key isn't found or if the value found isn't a valid integer.
///
/// # Examples
/// ```
/// # const LOG_DOMAIN: &str = "DOCTEST";
/// use dmsdk::*;
///
/// fn app_init(params: dmextension::AppParams) -> dmextension::Result {
///     let display_width = dmconfigfile::get_int(params.config, "display.width", 960);
///     dmlog::info!("Window width is: {display_width}");
///
///     dmextension::Result::Ok
/// }
/// ```
pub fn get_int(config: ConfigFile, key: &str, default_value: i32) -> i32 {
    let key = CString::new(key).unwrap();
    unsafe { dmConfigFile::GetInt(config.into(), key.as_ptr(), default_value) }
}

/// Gets the corresponding config value as an f32.
///
/// `default_value` will be returned instead if the key isn't found or if the value found isn't a valid float.
///
/// # Examples
/// ```
/// # const LOG_DOMAIN: &str = "DOCTEST";
/// use dmsdk::*;
///
/// fn app_init(params: dmextension::AppParams) -> dmextension::Result {
///     let gravity = dmconfigfile::get_float(params.config, "physics.gravity_y", -9.8);
///     dmlog::info!("Gravity is: {gravity}");
///
///     dmextension::Result::Ok
/// }
/// ```
pub fn get_float(config: ConfigFile, key: &str, default_value: f32) -> f32 {
    let key = CString::new(key).unwrap();
    unsafe { dmConfigFile::GetFloat(config.into(), key.as_ptr(), default_value) }
}

/// Callback function called during the config plugin lifecycle.
pub type PluginLifecycle = fn(ConfigFile);
/// Function used to provide config values.
pub type PluginGetter<T> = fn(ConfigFile, &str, T) -> Option<T>;
#[doc(hidden)]
pub type StringGetter = fn(ConfigFile, &str, &str) -> Option<String>;
#[doc(hidden)]
pub type RawPluginLifecycle = unsafe extern "C" fn(dmConfigFile::HConfig);
#[doc(hidden)]
pub type RawPluginGetter<T> =
    unsafe extern "C" fn(dmConfigFile::HConfig, *const c_char, T, *mut T) -> bool;
#[doc(hidden)]
pub type Desc = [u8; DESC_BUFFER_SIZE as usize];

#[doc(hidden)]
pub const DESC_BUFFER_SIZE: u32 = 64;

#[doc(hidden)]
#[macro_export]
macro_rules! declare_plugin_lifecycle {
    ($symbol:ident, $option:expr) => {
        #[no_mangle]
        unsafe extern "C" fn $symbol(config: dmconfigfile::RawConfigFile) {
            let func: Option<dmconfigfile::PluginLifecycle> = $option;
            if let Some(func) = func {
                func(config.into());
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_plugin_getter {
    ($symbol:ident, $option:expr, $type:ident) => {
        #[no_mangle]
        unsafe extern "C" fn $symbol(
            config: dmconfigfile::RawConfigFile,
            key: *const core::ffi::c_char,
            default_value: $type,
            out: *mut $type,
        ) -> bool {
            let func: Option<dmconfigfile::PluginGetter<$type>> = $option;
            if let Some(func) = func {
                let key = core::ffi::CStr::from_ptr(key)
                    .to_str()
                    .expect("Invalid UTF-8 sequence in key!");
                if let Some(value) = func(config.into(), key, default_value) {
                    out.write(value);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_plugin_string_getter {
    ($symbol:ident, $option:expr) => {
        #[no_mangle]
        unsafe extern "C" fn $symbol(
            config: dmconfigfile::RawConfigFile,
            key: *const core::ffi::c_char,
            default_value: *const core::ffi::c_char,
            out: *mut *const core::ffi::c_char,
        ) -> bool {
            let func: Option<dmconfigfile::StringGetter> = $option;
            if let Some(func) = func {
                let key = core::ffi::CStr::from_ptr(key).to_str();
                if key.is_err() {
                    dmlog::error!("Invalid UTF-8 sequence in key!");
                    return false;
                }

                let default_value = if default_value.is_null() {
                    ""
                } else {
                    match core::ffi::CStr::from_ptr(default_value).to_str() {
                        Ok(str) => str,
                        Err(_) => {
                            dmlog::error!("Invalid UTF-8 sequence in default value!");
                            return false;
                        }
                    }
                };

                if let Some(value) = func(config.into(), key.unwrap(), default_value) {
                    let cstr =
                        std::ffi::CString::new(value).expect("Unexpected null in return value!");

                    let boxed_str = Box::new(cstr);
                    out.write(Box::leak(boxed_str).as_ptr());
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    };
}

/// Equivalent to `DM_DECLARE_CONFIGFILE_EXTENSION` in regular C++ extensions.
///
/// Each `get` function is called whenever a config value is requested from Lua or C++.
/// Return [`Some`] to override a value with your own, or [`None`] to let another function handle it.
///
/// # Examples
/// ```
/// # const LOG_DOMAIN: &str = "DOCTEST";
/// use dmsdk::*;
///
/// fn plugin_create(config: dmconfigfile::ConfigFile) {
///     dmlog::info!("Config plugin created");
/// }
///
/// fn plugin_destroy(config: dmconfigfile::ConfigFile) {
///     dmlog::info!("Config plugin destroyed");
/// }
///
/// fn get_string(config: dmconfigfile::ConfigFile, key: &str, default_value: &str) -> Option<String> {
///     if key == "project.title" {
///         Some("My project now!".to_owned())
///     } else {
///         None
///     }
/// }
///
/// fn get_int(config: dmconfigfile::ConfigFile, key: &str, default_value: i32) -> Option<i32> {
///     if key == "custom_section.my_value" {
///         Some(123)
///     } else {
///         None
///     }
/// }
///
/// fn get_float(config: dmconfigfile::ConfigFile, key: &str, default_value: f32) -> Option<f32> {
///     Some(default_value * 10.0)
/// }
///
/// declare_configfile_extension!(
///     MY_CONFIG_PLUGIN,
///     Some(plugin_create),
///     Some(plugin_destroy),
///     Some(get_string),
///     Some(get_int),
///     Some(get_float)
/// );
/// ```
#[macro_export]
macro_rules! declare_configfile_extension {
    ($symbol:ident, $create:expr, $destroy:expr, $get_string:expr, $get_int:expr, $get_float:expr) => {
        paste! {
            static mut [<$symbol _PLUGIN_DESC>]: dmconfigfile::Desc = [0u8; dmconfigfile::DESC_BUFFER_SIZE as usize];

            declare_plugin_lifecycle!([<$symbol _plugin_create>], $create);
            declare_plugin_lifecycle!([<$symbol _plugin_destroy>], $destroy);
            declare_plugin_string_getter!([<$symbol _plugin_get_string>], $get_string);
            declare_plugin_getter!([<$symbol _plugin_get_int>], $get_int, i32);
            declare_plugin_getter!([<$symbol _plugin_get_float>], $get_float, f32);

            #[no_mangle]
            #[dmextension::ctor]
            unsafe fn $symbol() {
                dmconfigfile::register(
                    &mut [<$symbol _PLUGIN_DESC>],
                    stringify!($symbol),
                    [<$symbol _plugin_create>],
                    [<$symbol _plugin_destroy>],
                    [<$symbol _plugin_get_string>],
                    [<$symbol _plugin_get_int>],
                    [<$symbol _plugin_get_float>],
                );
            }
        }
    };
}

#[doc(hidden)]
pub fn register(
    desc: &mut Desc,
    name: &str,
    create: RawPluginLifecycle,
    destroy: RawPluginLifecycle,
    get_string: RawPluginGetter<*const c_char>,
    get_int: RawPluginGetter<i32>,
    get_float: RawPluginGetter<f32>,
) {
    let name = CString::new(name).unwrap();
    unsafe {
        dmsdk_ffi::ConfigFileRegisterExtension(
            desc.as_mut_ptr() as *mut c_void,
            DESC_BUFFER_SIZE,
            name.as_ptr(),
            Some(create),
            Some(destroy),
            Some(get_string),
            Some(get_int),
            Some(get_float),
        );
    }
}

#[doc(inline)]
pub use crate::declare_configfile_extension;
