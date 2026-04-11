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

TEST_FILES=$(find "${TEST_DIR}" -type f -name '*.class')

for TEST_FILE in ${TEST_FILES} ; do
    EXPECTED_OUTPUT=$(mktemp)
    ACTUAL_OUTPUT=$(mktemp)
    ${SYSTEM_JAVAP} -l -v -p "${TEST_FILE}" > "${EXPECTED_OUTPUT}"
    ${OUR_JAVAP} "${TEST_FILE}" > "${ACTUAL_OUTPUT}"
    diff "${EXPECTED_OUTPUT}" "${ACTUAL_OUTPUT}"
    printf "%s ... \033[31mOK\033[0m" "${TEST_FILE}"
done
