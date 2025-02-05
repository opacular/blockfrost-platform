#!/usr/bin/env bash

exe_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

PATH="$exe_dir/bundle/bin:$PATH"
export PATH

# Clear the terminal for nicer UX:
reset

color_bold=$'\e[1m'
color_reset=$'\e[0m'

cat <<EOF

  Welcome!

  This terminal is set up for easy execution of ‘${color_bold}blockfrost-platform${color_reset}’.

  Type one of the following commands:

    ${color_bold}blockfrost-platform --help${color_reset}
    ${color_bold}blockfrost-platform --init${color_reset}

  … and press <${color_bold}ENTER${color_reset}>.

  Documentation: https://platform.blockfrost.io

EOF

exec "$SHELL" -i
