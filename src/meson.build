global_conf = configuration_data()
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() /  'src' / 'config.rs',
  check: true
)

cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_build = custom_target(
    'cargo-build',
    build_by_default: true,
    install: true,
    install_dir: bindir,
    output: meson.project_name(),
    command: [
        'env',
        cargo_env,
        cargo, 'build',
        cargo_options,
        '&&',
        'cp', 'src' / rust_target / meson.project_name(), '@OUTPUT@'
    ]
)