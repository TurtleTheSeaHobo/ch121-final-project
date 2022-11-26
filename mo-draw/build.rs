use std::{ env, fs::File, path::PathBuf };
use cfg_aliases::cfg_aliases;
use gl_generator::{ Api, Fallbacks, Profile, Registry, GlobalGenerator };

fn main() {
    cfg_aliases! {
        android: { target_os = "android" },
        wasm: { target_arch = "wasm32" },
        macos: { target_os = "macos" },
        ios: { target_os = "ios" },
        apple: { any(target_os = "ios", target_os = "macos") },
        free_unix: { all(unix, not(apple), not(android)) },
        x11_platform: { all(feature = "x11", free_unix, not(wasm)) },
        wayland_platform: { all(feature = "wayland", free_unix, not(wasm)) },
        egl_backend: { all(feature = "egl", any(windows, unix), not(apple), not(wasm)) },
        glx_backend: { all(feature = "glx", x11_platform, not(wasm)) },
        wgl_backend: { all(feature = "wgl", windows, not(wasm)) },
        cgl_backend: { all(macos, not(wasm)) },
    }

    let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");

    let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, [])
             .write_bindings(GlobalGenerator, &mut file)
             .unwrap();
}
