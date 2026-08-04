defmodule ExLogFormatterTest do
  use ExUnit.Case, async: true
  doctest ExLogFormatter

  test "sanitizes ANSI escape sequences" do
    ansi_line = "\e[32m2026-08-04T13:14:00Z\e[0m [\e[31mERROR\e[0m] Test"
    clean = ExLogFormatter.sanitize(ansi_line)
    assert clean == "2026-08-04T13:14:00Z [ERROR] Test"
  end

  test "parses JSON log lines" do
    json_log = ~s({"level":"info","msg":"User login","user_id":42})
    {spans, is_error} = ExLogFormatter.format_line_with_meta(json_log)
    assert is_error == false
    assert length(spans) > 0
  end

  test "sub-highlights HTTP method, status, duration, IP and UUID" do
    line = "GET /api/v1/checkout 200 120ms 192.168.1.1 550e8400-e29b-44d4-a716-446655440000"
    spans = ExLogFormatter.sub_highlight(line)
    contents = Enum.map(spans, & &1.content)

    assert "GET" in contents
    assert "200" in contents
    assert "120ms" in contents
    assert "192.168.1.1" in contents
    assert "550e8400-e29b-44d4-a716-446655440000" in contents
  end

  test "processes chunk stream cleanly" do
    chunk = "line1\nline2\nline3_incomp"
    {complete, remaining} = ExLogFormatter.process_chunk(chunk, "")
    assert complete == ["line1", "line2"]
    assert remaining == "line3_incomp"
  end
end
