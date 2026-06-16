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
const TTD_SDK_PACKAGE_VERSION: &'static str = "0.9.5";

fn get_ttd_sdk_version() -> String {
    std::env::var("TTD_SDK_PACKAGE_VERSION").unwrap_or(TTD_SDK_PACKAGE_VERSION.to_string())
}

fn get_nuget_ttd_download_link() -> String {
    let package_version = get_ttd_sdk_version();
    format!("https://globalcdn.nuget.org/packages/{TTD_SDK_PACKAGE_NAME}.{package_version}.nupkg?packageVersion={package_version}",)
}

const WINGET_TTD_PACKAGE_NAME: &str = "Microsoft.TimeTravelDebugging";
const WINGET_TTD_PACKAGE_ID: &str = "8wekyb3d8bbwe";
const WINGET_TTD_PACKAGE_VERSION: &str = "1.11.584.0";

fn get_winget_ttd_package_version() -> String {
    std::env::var("WINGET_TTD_PACKAGE_VERSION").unwrap_or(WINGET_TTD_PACKAGE_VERSION.to_string())
}

fn get_winget_ttd_install_path() -> String {
    format!(
        "C:\\Program Files\\WindowsApps\\{}_{}_{}__{}",
        WINGET_TTD_PACKAGE_NAME,
        get_winget_ttd_package_version(),
        ARCH,
        WINGET_TTD_PACKAGE_ID
    )
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

fn get_ttd_ffi_build_dir() -> String {
    let base = get_ttd_ffi_base_dir();
    format!("{}/build", base.to_str().unwrap())
}

fn get_ttd_ffi_install_dir() -> String {
    let base = get_ttd_ffi_base_dir();
    format!("{}/install", base.to_str().unwrap())
}

fn get_ttd_ffi_install_include_dir() -> String {
    format!("{}/ttd_ffi/Include", get_ttd_ffi_install_dir())
}

fn get_ttd_ffi_install_library_dir() -> String {
    format!("{}/ttd_ffi/Library", get_ttd_ffi_install_dir())
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
    let ttd_rs_ffi_include_dir = get_ttd_ffi_install_include_dir();

    // CMake configure
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["-S", ttd_rs_ffi_base_dir.to_str().unwrap()])
                .args(["-B", ttd_rs_ffi_build_dir.as_str()])
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
                .args(["--build", ttd_rs_ffi_build_dir.as_str()])
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
                .args(["--install", ttd_rs_ffi_build_dir.as_str()])
                .args(["--config", BUILD_TYPE])
                .args(["--prefix", ttd_rs_ffi_install_library_dir.as_str()])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to install with cmake")
                .success()
        );
    }

    for hdr in ["ttd_ffi.cpp", "ttd_ffi.hpp", "constants.hpp.in"] {
        let header_path = format!("{}/{}", ttd_rs_ffi_include_dir.as_str(), hdr);
        // assert!(std::path::Path::new(header_path.as_str()).exists(), "{header_path} should exist but doesn't");
        println!("cargo:rerun-if-changed={}", header_path.as_str());
    }
}

fn generate_ttd_bindings() {
    let ttd_sdk_path = get_ttd_sdk_path();
    let ttd_ffi_install_dir = get_ttd_ffi_install_library_dir();

    if BUILD_TYPE == "Debug" {
        println!("cargo:rustc-link-lib=dylib=ucrtd");
    }

    // include libs
    {
        assert!(
            std::path::Path::new(ttd_ffi_install_dir.as_str()).exists(),
            "{ttd_ffi_install_dir} should exist but doesn't"
        );
        println!("cargo:rustc-link-search={}", ttd_ffi_install_dir.as_str());
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
        println!("cargo:rustc-link-search={}/{}", base_dir.to_str().unwrap(), ttd_ffi_install_dir);
        let src = std::path::PathBuf::from(format!("{}/{}", &inc, "ttd_ffi.hpp"));
        let dst = std::path::PathBuf::from("./src/bindings.rs");
        let bindings = bindgen::Builder::default()
            .generate_comments(true)
            .header(src.as_path().to_string_lossy())
            .clang_args(["-x", "c++", "-std=c++23"])
            .clang_arg(format!("-I{}", inc.as_str()))
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

    assert!(std::fs::exists(install_dir.as_str()).unwrap(), "{install_dir} doesn't exist but should");
}

fn copy_ttd_dlls() {
    let install_dir = get_winget_ttd_install_path();
    let ttd_dll_glob = format!("{install_dir}/*.dll");
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
        "./src/constants.rs",
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
    if !std::fs::exists(get_ttd_sdk_path()).unwrap() {
        download_nuget_package(TTD_SDK_PACKAGE_NAME, get_ttd_sdk_version().as_str());
    }

    for dll in TTD_DLLS {
        let path = std::path::PathBuf::from(format!("../{dll}"));
        if !std::fs::exists(&path).unwrap() {
            install_winget_ttd();
            copy_ttd_dlls();
            assert!(std::fs::exists(&path).unwrap(), "{} doesn't exist but should", &path.to_string_lossy());
        }
    }

    cmake_build_ffi();

    generate_ttd_bindings();

    generate_constant_file();
}
