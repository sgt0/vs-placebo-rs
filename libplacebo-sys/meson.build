project(
  'libplacebo-sys',
  'rust',
  version: '0.0.0',
  default_options: [
    'default_library=static',
    # 'c_std=c11',
    # 'cpp_std=c++20',
    'libplacebo:demos=false',
    'libplacebo:tests=false',
    # 'libplacebo:lcms=enabled',
    # 'libplacebo:libdovi=enabled',
    'libplacebo:d3d11=disabled',
    'libplacebo:glslang=disabled',
    'libplacebo:opengl=disabled',
    'libplacebo:shaderc=enabled',
    'libplacebo:vk-proc-addr=enabled',
    'libplacebo:vulkan=enabled',
    'libplacebo:default_library=static',
    'libplacebo:prefer_static=true',
  ],
)

# Use static versions of these libraries
# shaderc_combined = dependency('shaderc_combined')
# meson.override_dependency('shaderc', shaderc_combined)
# spirv_cross_c = dependency('spirv-cross-c')
# meson.override_dependency('spirv-cross-c-shared', spirv_cross_c)

placebo_proj = subproject('libplacebo')
placebo_dep = placebo_proj.get_variable('libplacebo')
# placebo_dep = dependency(
#   'libplacebo',
#   fallback: ['libplacebo', 'libplacebo'],
# )

# thirdparty = (meson.project_source_root() / 'subprojects/libplacebo/3rdparty')
# python = import('python').find_installation()
# python_env = environment()
# python_env.append('PYTHONPATH', thirdparty / 'jinja/src')

# rust_lib = static_library('rust_lib', sources: 'src/lib.rs', dependencies: deps)

rust = import('unstable-rust')
generated = rust.bindgen(
  input: 'wrapper.h',
  output: 'meson-bindings.rs',
  dependencies: [
    dependency('vulkan'),
    placebo_dep,
  ],
  args: [
    '--allowlist-item=PL_.*',
    '--allowlist-item=pl_.*',
    '--with-derive-default',
    '--default-enum-style=rust',
  ]
)

libplacebo_sys = shared_library(
  'libplacebo_sys',
  [generated],
  dependencies: placebo_dep.partial_dependency(links: true),
)

libplacebo_sys_dep = declare_dependency(link_with: libplacebo_sys)
