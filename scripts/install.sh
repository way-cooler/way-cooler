#!/bin/sh
set -eo pipefail
IFS=$'\n\t'

function assert_file_exists() {
    if ! [ -f "$1" ]; then
        echo -e "\e[31mCould not find file \"$1\", halting...\e[0m"
        exit 1
    fi
}

assert_file_exists "way-cooler"
assert_file_exists "way-cooler-bg"

if ! [[ $(id -u) = 0 ]] && ! [[ $# == 1 ]]; then
    echo -e "\e[31m"
    echo "The install script should be run as root!"
    echo -e "\e[0m \e[93m"
    echo "If you *really* want to install Way Cooler as a user, please pass an install path to this script, like so:"
    echo -e "\e[0m"
    echo "./install.sh ~/bin"
    echo -e "\e[31m"
    echo "It it highly discouraged to install Way Cooler as your regular user, as it makes it much less secure!"
    echo -e "\e[0m"
    exit 1
fi

install_path=$1;
: ${install_path:='/usr/bin'}
[ -d $install_path ] || mkdir $install_path

cp way-cooler $install_path
cp way-cooler-bg $install_path

chown $USER $install_path/way-cooler
chgrp $USER $install_path/way-cooler
chown $USER $install_path/way-cooler-bg
chgrp $USER $install_path/way-cooler-bg

if ! [[ $(pidof systemd) ]] && [[ $(id -u) = 0 ]]; then
    echo "systemd is not installed on this machine, activating the setuid bit on $install_path/way-cooler"
    chmod u+s $install_path/way-cooler
fi

echo -e "\e[32mWay Cooler has been installed on your system\e[0m"
