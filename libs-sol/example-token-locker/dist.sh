#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -e

# shellcheck disable=SC2045
for contract_path in $(ls ./contracts/*.sol)
do
  filename=$(basename -- "$contract_path")
  # Get contract name without extension and without directories.
  contract_name="${filename%.*}"
  npx truffle-flattener "./contracts/${contract_name}.sol" > "dist/${contract_name}.full.sol"
  npx solcjs --bin --abi --optimize "dist/${contract_name}.full.sol" -o "dist"
  mv "dist/dist_${contract_name}_full_sol_${contract_name}.abi" "dist/${contract_name}.full.abi"
  mv "dist/dist_${contract_name}_full_sol_${contract_name}.bin" "dist/${contract_name}.full.bin"
  rm -f dist/*_sol_*
done
