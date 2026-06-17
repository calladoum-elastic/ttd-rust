#![allow(dead_code)]
use std::path::PathBuf;

#[cfg(not(target_os = "windows"))]
const _: () = assert!(false, "TTD bindings only work on Windows");

#[cfg(debug_assertions)]
const BUILD_TYPE: &str = "Debug";
#[cfg(not(debug_assertions))]
const BUILD_TYPE: &str = "Release";

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "x64";

#[cfg(target_arch = "aarch64")]
const ARCH: &str = "arm64";

const TTD_SDK_PACKAGE_NAME: &str = "microsoft.timetraveldebugging.apis";
const TTD_SDK_PACKAGE_VERSION: &str = "0.9.5";

fn get_ttd_sdk_version() -> String {
    std::env::var("TTD_SDK_PACKAGE_VERSION").unwrap_or(TTD_SDK_PACKAGE_VERSION.to_string())
}

fn get_nuget_ttd_download_link() -> String {
    let package_version = get_ttd_sdk_version();
    format!("https://globalcdn.nuget.org/packages/{TTD_SDK_PACKAGE_NAME}.{package_version}.nupkg?packageVersion={package_version}",)
}

const WINGET_TTD_PACKAGE_NAME: &str = "Microsoft.TimeTravelDebugging";
const WINGET_TTD_PACKAGE_ID: &str = "8wekyb3d8bbwe";
const WINGET_TTD_PACKAGE_VERSION: &str = "1.11.553.0";

fn get_winget_ttd_package_version() -> String {
    std::env::var("WINGET_TTD_PACKAGE_VERSION").unwrap_or(WINGET_TTD_PACKAGE_VERSION.to_string())
}

fn get_winget_ttd_install_path() -> PathBuf {
    let mut path = PathBuf::from(r"C:\Program Files\WindowsApps");
    let fname = format!(
        "{}_{}_{}__{}",
        WINGET_TTD_PACKAGE_NAME,
        get_winget_ttd_package_version(),
        ARCH,
        WINGET_TTD_PACKAGE_ID
    );
    path.push(fname.as_str());
    path
}

fn get_package_root() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default())
        .parent()
        .unwrap()
        .to_path_buf()
}

fn get_ttd_base_dir() -> PathBuf {
    get_package_root().join("ttd")
}

fn get_ttd_ffi_base_dir() -> PathBuf {
    get_package_root().join("ttd_ffi")
}

fn get_ttd_sys_base_dir() -> PathBuf {
    get_package_root().join("ttd_sys")
}

fn get_ttd_ffi_build_dir() -> PathBuf {
    get_ttd_ffi_base_dir().join("build")
}

fn get_ttd_ffi_install_dir() -> PathBuf {
    get_ttd_ffi_base_dir().join("install")
}

fn get_ttd_ffi_install_include_dir() -> PathBuf {
    get_ttd_ffi_install_dir().join("ttd_ffi/Include")
}

fn get_ttd_ffi_install_library_dir() -> PathBuf {
    get_ttd_ffi_install_dir().join("ttd_ffi/Library")
}

const TTD_DLLS: [&str; 7] = [
    "TTDLiveRecorder.dll",
    "TTDLoader.dll",
    "TTDRecord.dll",
    "TTDRecordCPU.dll",
    "TTDRecordUI.dll",
    "TTDReplay.dll",
    "TTDReplayCPU.dll",
];

const NB_CPU: usize = 2;

fn get_nb_cpu() -> String {
    std::env::var("NUMBER_OF_PROCESSORS").unwrap_or(NB_CPU.to_string())
}

fn get_nuget_pkg_path() -> std::path::PathBuf {
    let mut nuget_path = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap().as_str());
    nuget_path.push(".nuget/packages");
    if !nuget_path.exists() {
        std::fs::create_dir_all(&nuget_path).unwrap();
        assert!(nuget_path.exists(), "{} doesn't exist but should", &nuget_path.to_string_lossy());
    }
    nuget_path
}

fn get_ttd_sdk_path() -> std::path::PathBuf {
    let mut ttd_sdk_path = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap().as_str());
    ttd_sdk_path.push(format!(".nuget/packages/{}/{}", TTD_SDK_PACKAGE_NAME, TTD_SDK_PACKAGE_VERSION).as_str());
    ttd_sdk_path
}

fn download_nuget_package(package_name: &str, version: &str) -> std::path::PathBuf {
    let download_link = get_nuget_ttd_download_link();
    let download_dir = get_nuget_pkg_path();
    std::fs::create_dir_all(&download_dir).unwrap();

    let package_path = download_dir.join(format!("{}.{}.nupkg", package_name, version));
    let extract_path = download_dir.join(format!("{}/{}", package_name, version));

    let mut response = reqwest::blocking::get(download_link.as_str()).unwrap();
    let mut dest_file = std::fs::File::create(&package_path).unwrap();
    std::io::copy(&mut response, &mut dest_file).unwrap();

    let package_file = std::fs::File::open(&package_path).unwrap();
    let mut archive = zip::ZipArchive::new(package_file).unwrap();

    archive.extract(&extract_path).unwrap();
    extract_path
}

fn cmake_build_ffi() {
    let ttd_rs_ffi_base_dir = get_ttd_ffi_base_dir();
    let ttd_rs_ffi_build_dir = get_ttd_ffi_build_dir();
    let ttd_rs_ffi_install_library_dir = get_ttd_ffi_install_library_dir();

    // CMake configure
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["-S", ttd_rs_ffi_base_dir.to_str().unwrap()])
                .args(["-B", ttd_rs_ffi_build_dir.to_str().unwrap()])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to configure cmake")
                .success()
        );
    }

    // CMake build
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["--build", ttd_rs_ffi_build_dir.to_str().unwrap()])
                .args(["--parallel", &get_nb_cpu()])
                .args(["--config", BUILD_TYPE])
                // .args(["--", "-D_LIBTTD_VERBOSE_OUTPUT"]) // Uncomment for verbose output from ttd_ffi
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to build with cmake")
                .success()
        );
    }

    // CMake install
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["--install", ttd_rs_ffi_build_dir.to_str().unwrap()])
                .args(["--config", BUILD_TYPE])
                .args(["--prefix", ttd_rs_ffi_install_library_dir.to_str().unwrap()])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to install with cmake")
                .success()
        );
    }

    // We need to watch changes on those specific files for recompilation
    let watched_files = [
        ttd_rs_ffi_base_dir.join("src/ttd_ffi.cpp"),
        ttd_rs_ffi_base_dir.join("src/constants.hpp.in"),
        ttd_rs_ffi_install_library_dir.join("ttd_ffi/Include/ttd_ffi.hpp"),
    ];

    for f in watched_files.as_ref() {
        assert!(f.exists(), "{} should exist but doesn't", f.to_string_lossy());
        println!("cargo:rerun-if-changed={}", f.to_string_lossy());
    }
}

fn generate_ttd_bindings() {
    let ttd_sdk_path = get_ttd_sdk_path();
    let ttd_ffi_install_lib_dir = get_ttd_ffi_install_library_dir();

    if BUILD_TYPE == "Debug" {
        println!("cargo:rustc-link-lib=dylib=ucrtd");
    }

    // include libs
    {
        assert!(
            ttd_ffi_install_lib_dir.exists(),
            "{} should exist but doesn't",
            ttd_ffi_install_lib_dir.to_string_lossy()
        );
        println!("cargo:rustc-link-search={}", ttd_ffi_install_lib_dir.to_string_lossy());
        println!("cargo:rustc-link-lib=ttd_ffi");

        let mut lib_path = ttd_sdk_path.clone();
        lib_path.push(format!("sdk/lib/{ARCH}"));

        println!("cargo:rustc-link-search={}", lib_path.as_path().to_string_lossy());
        for libname in ["TTDLiveRecorder", "TTDReplay"] {
            println!("cargo:rustc-link-lib={}", libname);
        }
    }

    // Create the binding files
    {
        let base_dir = get_ttd_sys_base_dir();
        let inc = get_ttd_ffi_install_include_dir();
        let ttd_sys_dir = get_ttd_sys_base_dir();
        println!(
            "cargo:rustc-link-search={}/{}",
            base_dir.to_str().unwrap(),
            ttd_ffi_install_lib_dir.to_str().unwrap()
        );
        let src = inc.join("ttd_ffi.hpp");
        let dst = ttd_sys_dir.join("src/bindings.rs");
        let bindings = bindgen::Builder::default()
            .generate_comments(true)
            .header(src.as_path().to_string_lossy())
            .clang_args(["-x", "c++", "-std=c++23"])
            .clang_arg(format!("-I{}", inc.to_str().unwrap()))
            .opaque_type("std::.*")
            .use_core()
            .opaque_type("TTD::TBufferView.*")
            .opaque_type("TTD::Replay::UniqueReplayEngine")
            .opaque_type("TTD::Replay::UniqueCursor")
            .blocklist_function("TTD::.*GetEndAddress.*")
            .allowlist_function("TTD_FFI::.*")
            .allowlist_type("TTD_FFI::.*")
            .allowlist_item("TTD_FFI::.*")
            .allowlist_var("TTD_FFI::.*")
            .allowlist_type("TTD::SystemInfo")
            .allowlist_type("TTD::Replay::Position")
            .enable_cxx_namespaces()
            .generate_inline_functions(true)
            .derive_default(true)
            .derive_debug(true)
            .derive_copy(true)
            .derive_hash(true)
            .derive_partialeq(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("bindgen failed");

        std::fs::write(
            dst.as_path(),
            format!(
                "//! Auto-generated bindings
#![allow(unused)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]

{}",
                bindings
            ),
        )
        .unwrap();
    }
}

fn install_winget_ttd() {
    let install_dir = get_winget_ttd_install_path();
    std::process::Command::new("winget")
        .args([
            "install",
            "--source",
            "winget",
            "--ignore-warnings",
            "--accept-package-agreements",
            "--silent",
            "--no-upgrade",
        ])
        .args(["--exact", "--id", WINGET_TTD_PACKAGE_NAME])
        .args(["--version", WINGET_TTD_PACKAGE_VERSION])
        .spawn()
        .unwrap()
        .wait()
        .expect("failed to install ttd");

    assert!(install_dir.exists(), "{} doesn't exist but should", install_dir.to_string_lossy());
}

fn copy_ttd_dlls() {
    let install_dir = get_winget_ttd_install_path();
    let ttd_dll_glob = format!("{}/*.dll", install_dir.to_string_lossy());
    for entry in glob::glob(&ttd_dll_glob).unwrap() {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.is_file() {
            continue;
        }

        let tgt = format!("../{}", entry.file_name().unwrap().to_string_lossy());
        std::fs::copy(&entry, tgt).unwrap();
    }
}

fn generate_constant_file() {
    std::fs::write(
        get_ttd_sys_base_dir().join("src/constants.rs"),
        format!(
            "//! Auto-generated constants
#![allow(unused)]

/// The `winget` package name for TTD
const TTD_PACKAGE_NAME: &str = \"{}\";

/// The `winget` package version for TTD
const TTD_PACKAGE_VERSION: &str = \"{}\";

",
            TTD_SDK_PACKAGE_NAME,
            get_ttd_sdk_version().as_str()
        ),
    )
    .unwrap();
}

fn main() {
    if !get_ttd_sdk_path().exists() {
        download_nuget_package(TTD_SDK_PACKAGE_NAME, get_ttd_sdk_version().as_str());
    }

    for dll in TTD_DLLS {
        let path = get_package_root().join(dll);
        if !path.exists() {
            install_winget_ttd();
            copy_ttd_dlls();
            assert!(path.exists(), "{} doesn't exist but should", path.to_string_lossy());
        }
    }

    cmake_build_ffi();

    generate_ttd_bindings();

    generate_constant_file();
}
