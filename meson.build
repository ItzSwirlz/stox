project('stox', 'rust',
  version : '0.1',
  default_options : ['warning_level=3'])

gnome = import('gnome')
i18n = import('i18n')

gtk4 = dependency('gtk4')

cargo = find_program('cargo', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
iconsdir = datadir / 'icons'
pkgdatadir = datadir / meson.project_name()
podir = meson.project_source_root () / 'po'
gettext_package = meson.project_name()

if get_option('profile') == 'development'
  profile = 'Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format (vcs_tag)
  endif
else
  profile = ''
  version_suffix = ''
endif

subdir('data')
subdir('po')
subdir('src')