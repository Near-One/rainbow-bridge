#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

yarn

# Remove existing files.
rm -f dist/*.sol

# shellcheck disable=SC2045
for contract_path in $(ls ./contracts/*.sol)
do
  filename=$(basename -- "$contract_path")
  # Get contract name without extension and without directories.
  contract_name="${filename%.*}"
  node_modules/.bin/truffle-flattener "./contracts/${contract_name}.sol" > "dist/${contract_name}.full.sol"
  # Fix for https://github.com/nomiclabs/truffle-flattener/issues/55
  if [[ "$(uname -s)" == "Darwin" ]];
  then
    # sed in-place has different behavior on mac: https://stackoverflow.com/a/62309999/4950797
    sed -i '' '/^\/\/ SPDX-License-Identifier:/d' "dist/${contract_name}.full.sol"
  else
    sed -i '/^\/\/ SPDX-License-Identifier:/d' "dist/${contract_name}.full.sol"
  fi
  yarn run solcjs --bin --abi --optimize "dist/${contract_name}.full.sol" -o "dist"
  mv "dist/dist_${contract_name}_full_sol_${contract_name}.abi" "dist/${contract_name}.full.abi"
  mv "dist/dist_${contract_name}_full_sol_${contract_name}.bin" "dist/${contract_name}.full.bin"
  rm -f dist/*_sol_*
done
