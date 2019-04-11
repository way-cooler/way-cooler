cargo = find_program('cargo', native: true)

custom_target(
    'way-cooler-client',
    command: [cargo, 'build', '--manifest-path=../client/Cargo.toml', '--target-dir=../build'],
    install: true,
    install_dir: 'way-cooler-client',
    output: 'way-cooler-client')
