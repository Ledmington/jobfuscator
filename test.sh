#!/usr/bin/env bash

set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <class_file>"
    exit 1
fi

INPUT=$(realpath "$1")

DIFF=$(realpath "$(which diff)")
JAVAP=$(realpath "$(which javap)")
JOBFUSCATOR=$(realpath ./target/debug/jobfuscator)

EXPECTED_OUTPUT=$(mktemp)
${JAVAP} -l -v -p "${INPUT}" > "${EXPECTED_OUTPUT}"

ACTUAL_OUTPUT=$(mktemp)
${JOBFUSCATOR} "${INPUT}" > "${ACTUAL_OUTPUT}"

${DIFF} "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}"
