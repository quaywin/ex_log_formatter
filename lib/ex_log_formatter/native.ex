defmodule ExLogFormatter.Native do
  @moduledoc false

  use Rustler, otp_app: :ex_log_formatter, crate: "ex_log_formatter_native"

  def sanitize_log(_line), do: :erlang.nif_error(:nif_not_loaded)
  def parse_log_line(_line), do: :erlang.nif_error(:nif_not_loaded)
  def sub_highlight_native(_text), do: :erlang.nif_error(:nif_not_loaded)
  def process_chunk_native(_chunk, _buffer, _max_len), do: :erlang.nif_error(:nif_not_loaded)
end
