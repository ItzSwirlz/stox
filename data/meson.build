install_data(
    '@0@.svg'.format('org.itzswirlz.stox'),
     install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps'
)

desktop_conf = configuration_data()
desktop_conf.set('icon', 'org.itzswirlz.stox')
desktop_file = i18n.merge_file (
  type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format('org.itzswirlz.stox'),
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format('org.itzswirlz.stox'),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'applications'
)
# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [
      desktop_file.full_path()
    ]
  )
endif

resources = gnome.compile_resources(
    'resources',
    'resources.gresource.xml',
    gresource_bundle: true,
    source_dir: meson.current_build_dir(),
    install: true,
    install_dir: pkgdatadir
)