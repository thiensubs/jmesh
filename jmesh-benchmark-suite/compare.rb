#!/usr/bin/env ruby
# compare.rb — reads results/{c,rust,python}.json and generates
# BENCHMARK_REPORT.md plus PNG charts in report/ (when gruff is available).
#
# Result JSON schema (per language):
#   {
#     "language": "Rust",
#     "library": "jmesh",
#     "benchmarks": {
#       "n1000": {
#         "import": { "records": 1000, "time_ms": 12.3, "cpu_user_ms": ...,
#                     "cpu_sys_ms": ..., "memory_kb": ..., "ops_per_sec": ... },
#         "export": { ... }
#       },
#       ...
#     }
#   }

require 'json'
require 'fileutils'
require 'time'

SUITE_DIR   = __dir__
RESULTS_DIR = File.join(SUITE_DIR, 'results')
REPORT_MD   = File.join(SUITE_DIR, 'BENCHMARK_REPORT.md')
CHART_DIR   = File.join(SUITE_DIR, 'report')

LANG_ORDER = %w[c rust python].freeze
LABELS = {
  'c'      => 'C (libsqlite3)',
  'rust'   => 'Rust (jmesh)',
  'python' => 'Python (sqlite-utils)',
}.freeze

# --- load results -----------------------------------------------------------

results = {}
LANG_ORDER.each do |lang|
  path = File.join(RESULTS_DIR, "#{lang}.json")
  next unless File.exist?(path)
  results[lang] = JSON.parse(File.read(path))
end

if results.empty?
  abort "no results found in #{RESULTS_DIR} — run `ruby run.rb` first"
end

scales = results.values.first['benchmarks'].keys.sort_by { |k| k.delete_prefix('n').to_i }

# --- charts (optional, needs gruff + ImageMagick) ---------------------------

charts = []
gruff_available = begin
  require 'gruff'
  true
rescue LoadError
  warn 'note: gruff not installed — skipping PNG charts (Markdown report still generated)'
  false
end

if gruff_available
  begin
    FileUtils.mkdir_p(CHART_DIR)

    {
      'import' => 'Import throughput (rows/sec, higher is better)',
      'export' => 'Export throughput (rows/sec, higher is better)',
    }.each do |op, title|
      g = Gruff::Bar.new(800)
      g.title = title
      g.hide_legend = false
      scales.each_with_index { |s, i| g.labels[i] = s.delete_prefix('n') }
      results.each do |lang, data|
        g.data(LABELS[lang], scales.map { |s| data['benchmarks'][s][op]['ops_per_sec'].to_f })
      end
      file = File.join(CHART_DIR, "#{op}_ops.png")
      g.write(file)
      charts << file
    end

    scales.each do |scale|
      g = Gruff::Bar.new(800)
      g.title = "Time at #{scale.delete_prefix('n')} rows (ms, lower is better)"
      # One bar group per language on the x-axis, import/export as series.
      g.labels = results.keys.each_with_index.to_h { |lang, i| [i, LABELS[lang]] }
      %w[import export].each do |op|
        g.data(op.capitalize, results.map { |lang, d| d['benchmarks'][scale][op]['time_ms'].to_f })
      end
      file = File.join(CHART_DIR, "time_#{scale}.png")
      g.write(file)
      charts << file
    end
  rescue StandardError => e
    warn "note: chart generation failed (#{e.message}) — skipping PNG charts"
    charts = []
  end
end

# --- markdown report --------------------------------------------------------

def fmt_int(n)
  n = n.to_i
  (n.negative? ? '-' : '') + n.abs.to_s.reverse.gsub(/(\d{3})(?=\d)/, '\\1,').reverse
end

def fmt_ms(ms)
  ms >= 1000 ? format('%.2f s', ms / 1000.0) : format('%.2f ms', ms)
end

lines = []
lines << '# Benchmark Report'
lines << ''
lines << "Generated: #{Time.now.utc.strftime('%Y-%m-%d %H:%M:%S UTC')}"
lines << ''
lines << '| Language | Library |'
lines << '|----------|---------|'
results.each { |lang, d| lines << "| #{LABELS[lang]} | `#{d['library']}` |" }
lines << ''

unless charts.empty?
  lines << '## Charts'
  lines << ''
  charts.each { |f| lines << "![#{File.basename(f, '.png')}](report/#{File.basename(f)})" }
  lines << ''
end

lines << '## Results'
scales.each do |scale|
  n = scale.delete_prefix('n').to_i
  lines << ''
  lines << "### #{fmt_int(n)} records"
  lines << ''
  lines << "| Language | Import time | Import rows/s | Export time | Export rows/s | Peak mem Δ (import) |"
  lines << '|----------|------------|---------------|-------------|---------------|---------------------|'
  results.each do |lang, d|
    b = d['benchmarks'][scale]
    lines << "| #{LABELS[lang]} " \
             "| #{fmt_ms(b['import']['time_ms'])} " \
             "| #{fmt_int(b['import']['ops_per_sec'])} " \
             "| #{fmt_ms(b['export']['time_ms'])} " \
             "| #{fmt_int(b['export']['ops_per_sec'])} " \
             "| #{fmt_int(b['import']['memory_kb'])} KB |"
  end
end

# Summary: winner per operation at the largest scale
largest = scales.last
lines << ''
lines << '## Summary'
lines << ''
lines << "Fastest at #{fmt_int(largest.delete_prefix('n').to_i)} records:"
lines << ''
%w[import export].each do |op|
  ranked = results.sort_by { |_, d| d['benchmarks'][largest][op]['time_ms'].to_f }
  winner_lang, winner = ranked.first
  lines << "- **#{op.capitalize}**: #{LABELS[winner_lang]} " \
           "(#{fmt_ms(winner['benchmarks'][largest][op]['time_ms'])}, " \
           "#{fmt_int(winner['benchmarks'][largest][op]['ops_per_sec'])} rows/s)"
  ranked[1..].each do |lang, d|
    slowdown = d['benchmarks'][largest][op]['time_ms'].to_f / winner['benchmarks'][largest][op]['time_ms'].to_f
    lines << "  - #{LABELS[lang]}: #{format('%.1f', slowdown)}× slower"
  end
end

rust = results['rust']
if rust && (stream = rust.dig('benchmarks', largest, 'export_stream'))
  materialized = rust['benchmarks'][largest]['export']['ops_per_sec'].to_f
  speedup = stream['ops_per_sec'].to_f / materialized
  lines << "- **Streaming export** (`write_jsonl`, jmesh only): " \
           "#{fmt_int(stream['ops_per_sec'])} rows/s " \
           "(#{format('%.1f', speedup)}× faster than materialized export)"
end
lines << ''

File.write(REPORT_MD, lines.join("\n"))
puts "wrote #{REPORT_MD}"
charts.each { |c| puts "wrote #{c}" }
