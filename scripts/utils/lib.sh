#!/usr/bin/env bash

# Log a message at the info level
function info {
    local message=$1
    echo -e "\033[32;1m[INFO]\033[0m $message" >&2
}

# Log a message at the warn level
function warn {
    local message=$1
    echo -e "\033[33;1m[WARN]\033[0m $message" >&2
}

# Log a message at the error level
function error {
    local message=$1
    echo -e "\033[31;1m[ERROR]\033[0m $message" >&2
}

# Log a message at the error level and exit the script with a non-zero exit code
function die {
    local message=$1
    error "$message"
    exit 1
}

# Log the command and execute it
function step {
    echo -e "\033[32;1m❱\033[0m $(colorize_command "$@")" >&2
    "$@"
}

# Returns a command with syntax highlighting
function colorize_command {
    local program=$1
    shift

    local args=()
    for arg in "$@"; do
        if [[ $arg =~ ^- ]]; then
            args+=("\033[34;1m${arg}\033[0m")
        else
            args+=("\033[0;33m${arg}\033[0m")
        fi
    done

    # On old versions of bash, for example 4.2.46 if the `args` array
    # is empty, then an `unbound variable` is thrown.
    #
    # Luckily, we don't pass commands without positional arguments to this function.
    # If this ever becomes a problem, you know the why and you'll hopefully fix it 😓.
    echo -e "\033[1;32m${program}\033[0m ${args[*]}"
}
