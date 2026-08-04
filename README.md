# ExLogFormatter ⚡

[![Hex.pm](https://img.shields.io/hexpm/v/ex_log_formatter.svg)](https://hex.pm/packages/ex_log_formatter)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ExLogFormatter** is a high-performance, zero-allocation Rust-accelerated log formatter, ANSI sanitizer, and sub-element highlighter for Elixir and [ex_ratatui](https://github.com/quaywin/ex_ratatui) TUI applications.

Powered by **Rust NIFs** via [Rustler](https://github.com/rusterlium/rustler), `ExLogFormatter` achieves **110,000 – 500,000 lines/second** throughput with up to **98% RAM reduction** compared to pure Elixir Regex parsers.

---

## 🚀 Key Features

- **⚡ Native Speed (Rust NIF):** Up to **514,000 lines/sec** for ANSI stripping and **123,000 lines/sec** for stream chunk processing.
- **🎨 Auto Sub-Highlighter:** Automatically highlights IPs, URLs, UUIDs, Durations, HTTP Methods, and Status Codes into `ExRatatui.Text.Span` structs.
- **🧹 Zero-Allocation ANSI Strip:** Nanosecond-level ANSI escape code removal via `strip-ansi-escapes`.
- **📦 Stream Chunk Splitter:** Memory-slice chunk splitting (`memchr`) for SSH and Socket streaming.
- **🛡️ Memory Safe:** Zero Erlang Heap garbage collection overhead during high-volume log streams.

---

## 📊 Benchmark Comparison Matrix

| Task | Elixir Pure Code | **ExLogFormatter (Rust NIF)** | Speedup | RAM Savings |
| :--- | :--- | :--- | :--- | :--- |
| **Sanitize ANSI Colors** | 225,070 ops/s | **514,170 ops/s** | 🚀 **2.3x Faster** | ⬇️ **-98% RAM (0.06 KB/line)** |
| **Process Chunk Stream** | 39,490 ops/s | **123,610 ops/s** | 🚀 **3.1x Faster** | ⬇️ **-90% RAM (1.50 KB/chunk)** |
| **Parse JSON Log** | 33,270 ops/s | **111,970 ops/s** | 🚀 **3.8x Faster** | ⬇️ **-52% RAM (5.61 KB/line)** |
| **Text Sub-Highlighting** | 18,590 ops/s | **49,050 ops/s** | 🚀 **2.6x Faster** | ⬇️ **-68% RAM (4.12 KB/line)** |

---

## 📥 Installation

Add `ex_log_formatter` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:ex_log_formatter, "~> 0.1.0"}
  ]
end
```

Or reference directly from GitHub:

```elixir
def deps do
  [
    {:ex_log_formatter, git: "https://github.com/quaywin/ex_log_formatter.git", branch: "main"}
  ]
end
```

---

## 💻 Usage

### 1. Sanitize ANSI Colors
```elixir
clean_text = ExLogFormatter.sanitize("\e[32m2026-08-04T13:14:00Z\e[0m [\e[31mERROR\e[0m] Crash")
# => "2026-08-04T13:14:00Z [ERROR] Crash"
```

### 2. Format Raw Log Line to Spans
```elixir
spans = ExLogFormatter.format_line("GET /api/v1/checkout 200 120ms 192.168.1.1")
# Returns a list of %ExRatatui.Text.Span{} structs ready for Ratatui TUI rendering
```

### 3. Stream Chunk Processing
```elixir
{complete_lines, remaining_buffer} = ExLogFormatter.process_chunk(chunk_binary, buffer_binary)
```

---

## 📄 License

ExLogFormatter is released under the **MIT License**.
