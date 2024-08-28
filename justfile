# list available recipes
_default:
	@just --list

# run all the tests and coverage
test:
	cargo nextest run --no-fail-fast
	cargo mutants
	cargo tarpaulin

# run all the fuzz targets with cargo fuzz
fuzz:
	cd fuzz/fuzz_targets && fd -e rs -x cargo fuzz run {.} -- -runs=100000
