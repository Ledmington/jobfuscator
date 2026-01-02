#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <class_file> [<class_file> ...]"
    exit 1
fi

DIFF=$(realpath "$(which diff)")
JAVAP=$(realpath "$(which javap)")
JOBFUSCATOR=$(realpath ./target/debug/javap)

GREEN='\033[0;32m'
RESET='\033[0m'

for arg in "$@"; do
    INPUT=$(realpath "$arg")

    EXPECTED_OUTPUT=$(mktemp)
    ACTUAL_OUTPUT=$(mktemp)

    "${JAVAP}" -l -v -p "${INPUT}" > "${EXPECTED_OUTPUT}"
    "${JOBFUSCATOR}" "${INPUT}" > "${ACTUAL_OUTPUT}"

    "${DIFF}" "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}"

    echo -e "${GREEN}OK${RESET} ${INPUT}"

    rm -f "$EXPECTED_OUTPUT" "$ACTUAL_OUTPUT"
done
