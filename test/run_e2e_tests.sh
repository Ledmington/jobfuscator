#!/bin/bash

set -euo pipefail

TEST_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
ROOT=$(realpath "${TEST_DIR}/..")

BUILD_MODE="${1:-debug}"
if [[ "${BUILD_MODE}" != "debug" && "${BUILD_MODE}" != "release" ]]; then
    echo "Usage: $0 [debug|release]"
    exit 1
fi

run_javap_test() {
    local TEST_NAME="$1"
    local TEST_FILE="${TEST_DIR}/data/${TEST_NAME}.class"
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

run_javap_tests() {
    SYSTEM_JAVAP=$(realpath "$(which javap)")
    OUR_JAVAP=$(realpath "${ROOT}/target/${BUILD_MODE}/javap")

    echo ""
    echo "End-to-End javap tests"
    echo " build mode:     ${BUILD_MODE}"
    echo " system's javap: ${SYSTEM_JAVAP}"
    echo " tested javap:   ${OUR_JAVAP}"
    echo ""

    set +e
    exit_code=0
    run_javap_test "HelloWorld" || exit_code=1
    run_javap_test "Math"       || exit_code=1
    run_javap_test "Stream"     || exit_code=1
    set -e

    if [ "$exit_code" -ne 0 ]; then
        exit $exit_code
    fi
}

run_roundtrip_test() {
    local TEST_NAME="$1"
    local TEST_FILE="${TEST_DIR}/data/${TEST_NAME}.class"
    local OUTPUT
    local EXPECTED_HEX
    local ACTUAL_HEX
    local DIFF_OUTPUT

    OUTPUT=$(mktemp)
    EXPECTED_HEX=$(mktemp)
    ACTUAL_HEX=$(mktemp)
    DIFF_OUTPUT=$(mktemp)

    "${JOBF}" --input "${TEST_FILE}" --output "${OUTPUT}" --quiet

    xxd "${TEST_FILE}" > "${EXPECTED_HEX}"
    xxd "${OUTPUT}" > "${ACTUAL_HEX}"

    diff "${EXPECTED_HEX}" "${ACTUAL_HEX}" > "${DIFF_OUTPUT}"
    local exit_code=$?

    if [ $exit_code -ne 0 ]; then
        echo -e "${TEST_FILE} ... \033[0;31mFAILED\033[0m"
        echo "Binary diff (xxd):"
        cat "${DIFF_OUTPUT}"
        return 1
    fi

    echo -e "${TEST_FILE} ... \033[0;32mOK\033[0m"

    rm -f "${OUTPUT}" "${EXPECTED_HEX}" "${ACTUAL_HEX}" "${DIFF_OUTPUT}"
}

run_roundtrip_tests() {
    JOBF=$(realpath "${ROOT}/target/${BUILD_MODE}/jobf")

    echo ""
    echo "End-to-End roundtrip parsing tests"
    echo " build mode:  ${BUILD_MODE}"
    echo " tested jobf: ${JOBF}"
    echo ""

    set +e
    exit_code=0
    run_roundtrip_test "HelloWorld" || exit_code=1
    run_roundtrip_test "Math"       || exit_code=1
    run_roundtrip_test "Stream"     || exit_code=1
    set -e

    if [ "$exit_code" -ne 0 ]; then
        exit $exit_code
    fi
}

run_javap_tests
run_roundtrip_tests

exit 0
