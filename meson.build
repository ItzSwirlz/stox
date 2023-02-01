project('stox', 'rust',
  version : '0.1',
  default_options : ['warning_level=3'])

i18n = import('i18n')

gtk4 = dependency('gtk4')

cargo = find_program('cargo', required: true)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

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

subdir('po')
subdir('src')