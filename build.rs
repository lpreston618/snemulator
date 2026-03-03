use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo where to find SDL3
    println!("cargo:rustc-link-search=vendored/SDL3/lib/x64");
    
    // Get the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .unwrap();
    
    // Copy SDL3.dll to the output directory
    let dll_src = "vendored/SDL3/lib/x64/SDL3.dll";
    let dll_dst = target_dir.join("SDL3.dll");
    
    if let Ok(_) = fs::copy(dll_src, &dll_dst) {
        println!("cargo:warning=Copied SDL3.dll to output directory");
    } else {
        println!("cargo:warning=Could not copy SDL3.dll - you may need to copy it manually");
    }
}