use libloading::{Library, Symbol};
use std::env;
use std::ffi::c_void;
use std::process;

fn call_dynamic_function(jvm_lib_path: &str, zip_lib_path: &str, func_name: &str, input_data: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {

    println!("call_dynamic_function: jvm_lib_path: {}", jvm_lib_path);
    let _libjvm = unsafe { Library::new(jvm_lib_path)? };
    println!("call_dynamic_function: libjvm loaded");

    println!("call_dynamic_function: zip_lib_path: {}", zip_lib_path);
    let libzip = unsafe { Library::new(zip_lib_path)? };
    println!("call_dynamic_function: libzip loaded");

    // The rest of the unsafe block
    unsafe {
        // Load the function by name
        let func: Symbol<unsafe extern fn(u32, *const c_void, u32) -> u32> = libzip.get(func_name.as_bytes())?;

        // Call the function inside the `unsafe` block
        let initial_crc = 0u32;

        // This is an unsafe function call, so it must be wrapped in `unsafe`.
        let result = func(initial_crc, input_data.as_ptr() as *const c_void, input_data.len() as u32);

        Ok(result)
    }
}

fn main() {
    // Path to the shared library (Java's libzip.so or zip.dll on Windows)
    let java_home;
    let jvm_lib_path:String;
    let zip_lib_path:String;

    match env::var("JAVA_HOME") {
        Ok(value) => {
            java_home = value;
        }
        Err(e) => { 
            println!("main: Couldn't read JAVA_HOME: {}", e); 
            process::exit(1); 
        },
    }

    match std::env::consts::OS {
        "windows" => {
            jvm_lib_path = format!("{}\\bin\\server\\jvm.dll", java_home);
            zip_lib_path = format!("{}\\bin\\zip.dll", java_home);
        },
        "macos" => {
            jvm_lib_path = format!("{}/lib/server/libjvm.dylib", java_home);
            zip_lib_path = format!("{}/lib/libzip.dylib", java_home);
        }
        _ => {
            jvm_lib_path = format!("{}/lib/server/libjvm.so", java_home);
            zip_lib_path = format!("{}/lib/libzip.so", java_home);
        },
    }

    // The function name we want to call (e.g., "crc32")
    let func_name = "crc32";

    // Example input data to pass to the function
    let str = "Mary had a little lamb whose fleece was white as snow!";
    println!("Let's hash the following with Java's CRC32.update function: {}", str);
    let bytes=str.as_bytes();

    // Observed result.
    let observed:u32;

    // Call the dynamic function.
    match call_dynamic_function(&jvm_lib_path, &zip_lib_path, func_name, bytes) {
        Ok(result) => {
            println!("main: Function {} returned: {:08x}", func_name, result);
            observed = result;
        },
        Err(e) => {
            eprintln!("main: Error: {}", e);
            process::exit(1); 
        },
    };
    let expected:u32 = 0xc2d80bc5;
    assert_eq!(observed, expected, "{:08x} = {:08x} ?", observed, expected);
}

