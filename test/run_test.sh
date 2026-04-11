#!/bin/bash

set -euo pipefail

TEST_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
ROOT=$(realpath "${TEST_DIR}/..")

BUILD_MODE="${1:-debug}"
if [[ "${BUILD_MODE}" != "debug" && "${BUILD_MODE}" != "release" ]]; then
    echo "Usage: $0 [debug|release]"
    exit 1
fi

SYSTEM_JAVAP=$(realpath "$(which javap)")
OUR_JAVAP=$(realpath "${ROOT}/target/${BUILD_MODE}/javap")

echo ""
echo "jObfuscator Integration tests"
echo " build mode:     ${BUILD_MODE}"
echo " system's javap: ${SYSTEM_JAVAP}"
echo " tested javap:   ${OUR_JAVAP}"
echo ""

run_test() {
    local TEST_NAME="$1"
    local TEST_FILE="${TEST_DIR}/${TEST_NAME}.class"
    local EXPECTED_OUTPUT
    local ACTUAL_OUTPUT
    local DIFF_OUTPUT

    EXPECTED_OUTPUT=$(mktemp)
    ACTUAL_OUTPUT=$(mktemp)
    DIFF_OUTPUT=$(mktemp)

    ${SYSTEM_JAVAP} -l -v -p "${TEST_FILE}" > "${EXPECTED_OUTPUT}"
    ${OUR_JAVAP} "${TEST_FILE}" > "${ACTUAL_OUTPUT}"

    diff "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}" > "${DIFF_OUTPUT}"
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo -e "${TEST_FILE} ... \033[0;31mFAILED\033[0m"
        echo "Expected output: ${EXPECTED_OUTPUT}"
        echo "Actual output: ${ACTUAL_OUTPUT}"
        cat "${DIFF_OUTPUT}"
        return 1
    fi

    echo -e "${TEST_FILE} ... \033[0;32mOK\033[0m"
    rm -f "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}" "${DIFF_OUTPUT}"
}

run_test "HelloWorld"
run_test "Math"
