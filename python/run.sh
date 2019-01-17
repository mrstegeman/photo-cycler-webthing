#!/bin/bash

dir=$(readlink -f $(dirname "$0"))
cd "${dir}"

system_option=""
if [ -f /etc/debian_version ]; then
    system_option="--system"
fi

if [ ! -d lib ]; then
    pip3 install $system_option --install-option="--prefix=" --target lib -r requirements.txt
fi

PYTHONPATH=./lib python3 photo-cycler-webthing.py "${dir}/../photos" "${dir}/../static"
