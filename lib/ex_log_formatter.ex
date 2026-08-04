defmodule ExLogFormatter do
  @moduledoc """
  High-performance, Rust-accelerated log formatter and sanitizer for Elixir and Ratatui TUI applications.
  """

  alias ExLogFormatter.Native

  @doc """
  Sanitizes ANSI escape sequences from a log string using native Rust memory scanning.
  """
  def sanitize(line) when is_binary(line) do
    Native.sanitize_log(line)
  end

  @doc """
  Parses a raw log line into a tuple `{spans, is_error}`.
  Supports JSON, Logfmt, and Text with auto-sub-highlighting.
  """
  def format_line_with_meta(line) when is_binary(line) do
    Native.parse_log_line(line)
  end

  @doc """
  Formats a raw log line into a list of colored `ExRatatui.Text.Span` structs.
  """
  def format_line(line) when is_binary(line) do
    {spans, _is_error} = Native.parse_log_line(line)
    spans
  end

  @doc """
  High-speed sub-element highlighter for IP addresses, URLs, UUIDs, Durations, HTTP Methods, and Status Codes.
  """
  def sub_highlight(text) when is_binary(text) do
    Native.sub_highlight_native(text)
  end

  @doc """
  Zero-allocation binary chunk splitter for log streams over SSH or Sockets.
  Returns `{complete_lines, remaining_buffer}`.
  """
  def process_chunk(chunk, buffer, max_size \\ 5000) when is_binary(chunk) and is_binary(buffer) do
    Native.process_chunk_native(chunk, buffer, max_size)
  end
end
