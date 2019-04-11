cargo = find_program('cargo', native: true)

custom_target(
    'awesome',
    command: [cargo, 'build', '--manifest-path=../awesome/Cargo.toml', '--target-dir=../build'],
    install: true,
    install_dir: 'aoeu',
    output: 'aoeu')
