project(
    'way-cooler',
    'c',
    version: '0.8.0',
    license: 'MIT',
    meson_version: '>=0.48.0',
    default_options: [
        'c_std=c11',
        'warning_level=2',
        'werror=true',
    ],
)

add_project_arguments(
    [
        '-DWLR_USE_UNSTABLE',

        '-Wno-unused-parameter',
        '-Wno-unused-result',
        '-Wundef',
        '-Wvla',
    ],
    language: 'c'
)

cc = meson.get_compiler('c')

wayland_server = dependency('wayland-server')
wayland_client = dependency('wayland-client')
wayland_cursor = dependency('wayland-cursor')
wayland_egl    = dependency('wayland-egl')
wayland_protos = dependency('wayland-protocols', version: '>=1.14')
xkbcommon      = dependency('xkbcommon')
xcb            = dependency('xcb', required: true)
git            = find_program('git', native: true, required: false)

# Try first to find wlroots as a subproject, then as a system dependency
wlroots_version = '>=0.5.0'
wlroots_proj = subproject(
	'wlroots',
	default_options: ['rootston=false', 'examples=false'],
	required: false,
	version: wlroots_version,
)
if wlroots_proj.found()
	wlroots = wlroots_proj.get_variable('wlroots')
	wlroots_conf = wlroots_proj.get_variable('conf_data')
	wlroots_has_xwayland = wlroots_conf.get('WLR_HAS_XWAYLAND') == 1
else
	wlroots = dependency('wlroots', version: wlroots_version)
	wlroots_has_xwayland = cc.get_define('WLR_HAS_XWAYLAND', prefix: '#include <wlr/config.h>', dependencies: wlroots) == '1'
endif

if not wlroots_has_xwayland
	error('Cannot build way-cooler: wlroots has been built without Xwayland support')
endif

if git.found()
	git_commit_hash = run_command([git.path(), 'describe', '--always', '--tags'])
	git_branch = run_command([git.path(), 'rev-parse', '--abbrev-ref', 'HEAD'])
	if git_commit_hash.returncode() == 0 and git_branch.returncode() == 0
		version = '"@0@ (" __DATE__ ", branch \'@1@\')"'.format(git_commit_hash.stdout().strip(), git_branch.stdout().strip())
	endif
endif
add_project_arguments('-DWAY_COOLER_VERSION=@0@'.format(version), language: 'c')

way_cooler_deps = [
	wayland_server,
	wlroots,
	xkbcommon,
]

way_cooler_sources = files(
    'main.c',
)

executable(
    'way-cooler-compositor',
    way_cooler_sources,
    include_directories: [include_directories('.')],
	dependencies: way_cooler_deps,
)
