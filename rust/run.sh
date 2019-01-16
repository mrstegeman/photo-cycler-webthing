#!/bin/bash

dir=$(readlink -f $(dirname "$0"))
cd "${dir}"

cargo run -- "${dir}/../photos" "${dir}/../static"
