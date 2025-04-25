extern crate dunce;

use dunce::canonicalize;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  let build_path = out_path.join("build");

  if cfg!(feature = "vendored") {
    println!("cargo::rerun-if-changed=./subprojects");

    run_meson(".", &build_path, true);
    let build_out_path = build_path.join("subprojects/libplacebo/src"); // libplacebo/build/src

    // fs::copy(
    //   build_out_path.join("libplacebo.a"),
    //   build_out_path.join("placebo.lib"),
    // )
    // .expect("Unable to copy `libplacebo.a` to `placebo.lib`");

    println!(
      "cargo::warning=build_out_path = {}",
      build_out_path.to_str().unwrap()
    );

    println!(
      "cargo::rustc-link-search=native={}",
      // libdir_path.join("lib").to_str().unwrap()
      build_out_path.to_str().unwrap()
    );

    // println!("cargo::rustc-link-lib=static=placebo");
    // println!("cargo:rustc-link-lib=placebo");

    // println!("cargo:rustc-link-lib=m");
    // println!("cargo:rustc-link-lib=version");
    // println!("cargo:rustc-link-lib=vulkan-1");
    // println!("cargo:rustc-link-lib=lcms2");
    // println!("cargo:rustc-link-lib=lcms2_fast_float");
    // println!("cargo:rustc-link-lib=shlwapi");
    // println!("cargo:rustc-link-lib=MachineIndependent");
    // println!("cargo:rustc-link-lib=OSDependent");
    // println!("cargo:rustc-link-lib=GenericCodeGen");
    // println!("cargo:rustc-link-lib=glslang");
    // println!("cargo:rustc-link-lib=SPIRV");

    // println!("cargo:rustc-link-lib=SPIRV-Tools");
    // println!("cargo:rustc-link-lib=SPIRV-Tools-opt");

    // println!("cargo:rustc-link-lib=SPIRV-Tools-link");
    // println!("cargo:rustc-link-lib=spirv-cross-glsl");
    // println!("cargo:rustc-link-lib=shaderc_combined");
    // println!("cargo:rustc-link-lib=spirv-cross-c");
    // println!("cargo:rustc-link-lib=spirv-cross-c-shared");
    // println!("cargo:rustc-link-lib=shaderc_shared");

    // println!("cargo:rustc-link-lib=stdc++");
    // println!("cargo:rustc-link-lib=user32");

    // println!("cargo:rustc-link-lib=kernel32");
    // println!("cargo:rustc-link-lib=dovi");
    // println!("cargo:rustc-link-lib=shaderc_combined");
  }

  // #[cfg(target_env = "gnu")]
  // println!("cargo::rustc-link-lib=dylib=stdc++");
  // println!("cargo::rustc-link-lib=static:-bundle=stdc++");
  // println!("cargo::rustc-link-lib=static=shlwapi");
  // println!("cargo::rustc-link-lib=static=shaderc_combined");
  // println!("cargo::rustc-link-lib=vulkan-1");
  // println!("cargo::rustc-link-lib=static=placebo");

  let mut builder = bindgen::Builder::default()
    .clang_arg("--verbose")
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .allowlist_item("PL_.*")
    .allowlist_item("pl_.*")
    .blocklist_item("pl_shader_obj_t")
    .default_enum_style(bindgen::EnumVariation::Rust {
      non_exhaustive: false,
    })
    .derive_default(true);

  if cfg!(feature = "vendored") {
    let install_path = canonicalize(build_path.join("install")).unwrap();
    let libplacebo_headers_path = canonicalize(install_path.join("include")).unwrap();

    builder = builder
      .clang_arg("--include-directory")
      .clang_arg(libplacebo_headers_path.to_str().unwrap());
  }

  let bindings = builder.generate().expect("Unable to generate bindings");

  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");

  print_link_flags();
}

fn run_meson<L, D>(lib: L, dir: D, static_linking: bool)
where
  L: AsRef<OsStr>,
  D: AsRef<OsStr>,
{
  if !is_configured(dir.as_ref()) {
    run_command(
      lib,
      "meson",
      &[
        OsStr::new("setup"),
        OsStr::new("."),
        dir.as_ref(),
        OsStr::new("--debug"),
        OsStr::new("--default-library"),
        OsStr::new(if static_linking { "static" } else { "shared" }),
        OsStr::new("--prefer-static"),
        // OsStr::new("-Dcpp_link_args='-static'"),
        OsStr::new(
          format!(
            "-Dprefix={}",
            PathBuf::from(dir.as_ref())
              .join("install")
              .to_str()
              .unwrap()
          )
          .as_str(),
        ),
        // OsStr::new("--cross-file"),
        // OsStr::new("x86_64-w64-mingw32.meson"),
        OsStr::new("-Dlibplacebo:demos=false"),
        OsStr::new("-Dlibplacebo:tests=false"),
        OsStr::new("-Dlibplacebo:glslang=disabled"),
        OsStr::new("-Dlibplacebo:shaderc=enabled"),
      ],
    );
  }
  run_command(dir, "meson", &[OsStr::new("install")]);
}

fn run_command<D, N>(dir: D, name: N, args: &[&OsStr])
where
  D: AsRef<OsStr>,
  N: AsRef<OsStr>,
{
  let mut cmd = Command::new(name);
  cmd.current_dir(dir.as_ref());
  if !args.is_empty() {
    cmd.args(args);
  }
  let out = match cmd.output() {
    Ok(v) => v,
    Err(err) => panic!("unable to invoke {:?}: {}", cmd, err),
  };
  if !out.status.success() {
    // This does not work great on Windows with non-ascii output,
    // but for now it"s good enough.
    let errtext = String::from_utf8_lossy(&out.stderr);
    let outtext = String::from_utf8_lossy(&out.stdout);
    panic!("{:?} invocation failed:\n{}\n{}", cmd, outtext, errtext);
  }
}

fn is_configured<S>(dir: S) -> bool
where
  S: AsRef<OsStr>,
{
  let mut path = PathBuf::from(dir.as_ref());
  path.push("build.ninja");
  path.exists()
}

fn print_link_flags() {
  // libplacebo deps.
  println!("cargo::rustc-link-lib=m");
  println!("cargo::rustc-link-lib=shaderc_combined");
  // println!("cargo::rustc-link-lib=static=glslang");
  // println!("cargo::rustc-link-lib=static=spirv-cross-c");
  println!("cargo::rustc-link-lib=vulkan-1");

  if let Ok(stdlib) = env::var("CXXSTDLIB") {
    if !stdlib.is_empty() {
      println!("cargo::rustc-link-lib=dylib={}", stdlib);
    }
  } else {
    let target = env::var("TARGET").unwrap();
    if target.contains("msvc") {
      // Nothing to link to.
    } else if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
      println!("cargo::rustc-link-lib=dylib=c++");
    } else if target.contains("android") {
      println!("cargo::rustc-link-lib=dylib=c++_shared");
    } else {
      // println!("cargo::rustc-link-lib=stdc++");
    }
  }

  let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
  if target_os == "windows" {
    println!("cargo::rustc-link-lib=dylib=dbghelp");
    println!("cargo::rustc-link-lib=dylib=winmm");

    println!("cargo::rustc-link-lib=dylib=shlwapi");
  }

  println!("cargo::rustc-link-lib=static=placebo");
}
