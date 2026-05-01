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
    run_javap_test "HelloWorld"          || exit_code=1
    run_javap_test "Math"                || exit_code=1
    run_javap_test "Stream"              || exit_code=1
    run_javap_test "List"                || exit_code=1
    run_javap_test "TimeUnit"            || exit_code=1
    run_javap_test "Arrays"              || exit_code=1
    run_javap_test "SecuritySettings\$1" || exit_code=1
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

    "${JOBF}" --input "${TEST_FILE}" --output "${OUTPUT}" --quiet=true --force

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
    run_roundtrip_test "HelloWorld"          || exit_code=1
    run_roundtrip_test "Math"                || exit_code=1
    run_roundtrip_test "Stream"              || exit_code=1
    run_roundtrip_test "List"                || exit_code=1
    run_roundtrip_test "TimeUnit"            || exit_code=1
    run_roundtrip_test "Arrays"              || exit_code=1
    run_roundtrip_test "SecuritySettings\$1" || exit_code=1
    set -e

    if [ "$exit_code" -ne 0 ]; then
        exit $exit_code
    fi
}

run_field_shuffle_test() {
    local TEST_NAME="$1"
    local TEST_FILE="${TEST_DIR}/data/${TEST_NAME}.class"
    local TEMP_FILE
    local EXPECTED_OUTPUT
    local ACTUAL_OUTPUT
    local DIFF_OUTPUT

    TEMP_FILE=$(mktemp --suffix=.class)
    EXPECTED_OUTPUT=$(mktemp)
    ACTUAL_OUTPUT=$(mktemp)
    DIFF_OUTPUT=$(mktemp)

    ${JOBF} --input "${TEST_FILE}" --output "${TEMP_FILE}" --quiet=true --seed=0x01020304 --shuffle-fields=true --force

    ${SYSTEM_JAVAP} -l -v -p "${TEMP_FILE}" > "${EXPECTED_OUTPUT}"
    ${OUR_JAVAP} "${TEMP_FILE}" > "${ACTUAL_OUTPUT}"

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

run_field_shuffle_tests() {
    SYSTEM_JAVAP=$(realpath "$(which javap)")
    OUR_JAVAP=$(realpath "${ROOT}/target/${BUILD_MODE}/javap")
    JOBF=$(realpath "${ROOT}/target/${BUILD_MODE}/jobf")

    echo ""
    echo "Field-shuffle tests"
    echo " build mode:     ${BUILD_MODE}"
    echo " system's javap: ${SYSTEM_JAVAP}"
    echo " javap:          ${OUR_JAVAP}"
    echo " tested jobf:    ${JOBF}"
    echo ""

    set +e
    exit_code=0
    run_field_shuffle_test "HelloWorld"          || exit_code=1
    run_field_shuffle_test "Math"                || exit_code=1
    run_field_shuffle_test "Stream"              || exit_code=1
    run_field_shuffle_test "List"                || exit_code=1
    run_field_shuffle_test "TimeUnit"            || exit_code=1
    run_field_shuffle_test "Arrays"              || exit_code=1
    run_field_shuffle_test "SecuritySettings\$1" || exit_code=1
    set -e

    if [ "$exit_code" -ne 0 ]; then
        exit $exit_code
    fi
}

run_execution_test() {
    local TEST_NAME="$1"
    local TEST_FILE="${TEST_DIR}/data/${TEST_NAME}.class"
    local TEMP_DIR
    local TEMP_FILE
    local EXPECTED_OUTPUT
    local ACTUAL_OUTPUT
    local DIFF_OUTPUT

    TEMP_DIR=$(mktemp -d)
    TEMP_FILE="${TEMP_DIR}/${TEST_NAME}.class"
    EXPECTED_OUTPUT=$(mktemp)
    ACTUAL_OUTPUT=$(mktemp)
    DIFF_OUTPUT=$(mktemp)

    # Run original class and capture output; fail if not executable
    ${SYSTEM_JAVA} -cp "${TEST_DIR}/data" "${TEST_NAME}" > "${EXPECTED_OUTPUT}" 2>&1
    local java_exit_code=$?
    if [ $java_exit_code -ne 0 ]; then
        echo -e "${TEST_FILE} ... \033[0;31mFAILED\033[0m (original class is not executable, exit code: ${java_exit_code})"
        cat "${EXPECTED_OUTPUT}"
        rm -f "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}" "${DIFF_OUTPUT}"
        rm -rf "${TEMP_DIR}"
        return 1
    fi

    "${JOBF}" --input "${TEST_FILE}" --output "${TEMP_FILE}" --quiet=true --seed=0x01020304 --shuffle-fields=true --force
    ${SYSTEM_JAVA} -cp "${TEMP_DIR}" "${TEST_NAME}" > "${ACTUAL_OUTPUT}" 2>&1

    diff "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}" > "${DIFF_OUTPUT}"
    local exit_code=$?

    if [ $exit_code -ne 0 ]; then
        echo -e "${TEST_FILE} ... \033[0;31mFAILED\033[0m"
        echo "Expected output: ${EXPECTED_OUTPUT}"
        echo "Actual output: ${ACTUAL_OUTPUT}"
        cat "${DIFF_OUTPUT}"
        rm -rf "${TEMP_DIR}"
        return 1
    fi

    echo -e "${TEST_FILE} ... \033[0;32mOK\033[0m"
    rm -f "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}" "${DIFF_OUTPUT}"
    rm -rf "${TEMP_DIR}"
}

run_execution_tests() {
    SYSTEM_JAVA=$(realpath "$(which java)")
    JOBF=$(realpath "${ROOT}/target/${BUILD_MODE}/jobf")

    echo ""
    echo "End-to-End execution tests (field-shuffle must preserve behaviour)"
    echo " build mode:     ${BUILD_MODE}"
    echo " system's java:  ${SYSTEM_JAVA}"
    echo " tested jobf:    ${JOBF}"
    echo ""

    set +e
    exit_code=0
    run_execution_test "HelloWorld"          || exit_code=1
    set -e

    if [ "$exit_code" -ne 0 ]; then
        exit $exit_code
    fi
}

run_javap_tests
run_roundtrip_tests
run_field_shuffle_tests
run_execution_tests

exit 0
