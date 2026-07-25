#!/usr/bin/env ruby
# run.rb — builds and runs the C, Rust, and Python benchmarks, collects
# their results into results/, then generates the report via compare.rb.
#
# Usage:
#   ruby run.rb                 # run all benchmarks + report
#   ruby run.rb c rust          # run only the listed benchmarks + report
#   ruby run.rb report          # skip benchmarks, only regenerate the report

require 'fileutils'

SUITE_DIR   = __dir__
RESULTS_DIR = File.join(SUITE_DIR, 'results')
LANGS       = %w[python c rust].freeze

FileUtils.mkdir_p(RESULTS_DIR)

def sh(cmd, dir:)
  puts "==> #{cmd}   (#{dir})"
  system(cmd, chdir: dir) or abort "FAILED: #{cmd}"
end

selected = ARGV.empty? ? LANGS : ARGV

unless (selected - LANGS - %w[report]).empty?
  abort "usage: ruby run.rb [#{LANGS.join('|')}|report] ..."
end

selected.each do |lang|
  case lang
  when 'python'
    dir = File.join(SUITE_DIR, 'python')
    sh('python3 bench.py', dir: dir)
    FileUtils.cp(File.join(dir, 'results.json'), File.join(RESULTS_DIR, 'python.json'))
  when 'c'
    dir = File.join(SUITE_DIR, 'c')
    sh('gcc -O3 -o bench bench.c -lsqlite3 -lm', dir: dir)
    sh('./bench', dir: dir)
    FileUtils.cp(File.join(dir, 'results.json'), File.join(RESULTS_DIR, 'c.json'))
  when 'rust'
    dir = File.join(SUITE_DIR, 'rust')
    sh('cargo build --release', dir: dir)
    sh('./target/release/bench', dir: dir)
    FileUtils.cp(File.join(dir, 'results.json'), File.join(RESULTS_DIR, 'rust.json'))
  when 'report'
    # handled below
  end
end

sh("ruby #{File.join(SUITE_DIR, 'compare.rb')}", dir: SUITE_DIR)
