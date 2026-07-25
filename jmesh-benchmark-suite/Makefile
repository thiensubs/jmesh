.PHONY: all python c rust report clean

all:
	ruby run.rb

python:
	ruby run.rb python

c:
	ruby run.rb c

rust:
	ruby run.rb rust

report:
	ruby run.rb report

clean:
	rm -f c/bench c/results.json python/results.json rust/results.json
	rm -f results/*.json report/*.png BENCHMARK_REPORT.md
	rm -rf rust/target
