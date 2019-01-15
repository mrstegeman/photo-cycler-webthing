#!/bin/bash

set -e

dir=$(readlink -f $(dirname "$0"))
cd "${dir}"

mvn clean compile assembly:single
jar=$(find target -maxdepth 1 -name '*-with-dependencies.jar')
java -jar "${jar}" "${dir}/../photos" "${dir}/../static"
