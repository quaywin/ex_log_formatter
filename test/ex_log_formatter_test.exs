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

  test "parses Logfmt key=value streams" do
    logfmt = "time=2026-08-04T13:14:00Z level=info msg=\"User logged in\" ip=192.168.1.10 status=200"
    {spans, is_error} = ExLogFormatter.format_line_with_meta(logfmt)
    assert is_error == false
    assert length(spans) > 0
    contents = Enum.map(spans, & &1.content)
    assert "time=" in contents
    assert "[INFO]" in contents
  end

  test "parses traditional bracket level text logs" do
    text = "2026-08-04T13:14:00Z [ERROR] Database connection refused on port 5432"
    {spans, is_error} = ExLogFormatter.format_line_with_meta(text)
    assert is_error == true
    assert length(spans) > 0
  end

  test "parses docker log timestamps accurately" do
    line = "2026-08-04T13:14:00.123456789Z stdout F Application ready on port 4000"
    {ts, msg} = ExLogFormatter.parse_docker_log(line)
    assert ts == "2026-08-04T13:14:00.123456789Z"
    assert msg == "stdout F Application ready on port 4000"

    non_docker = "Plain log line without timestamp"
    {ts_nil, msg_raw} = ExLogFormatter.parse_docker_log(non_docker)
    assert ts_nil == nil
    assert msg_raw == "Plain log line without timestamp"
  end

  test "wraps spans correctly by column width" do
    {spans, _} = ExLogFormatter.format_line_with_meta("1234567890abcdefghij")
    wrapped = ExLogFormatter.wrap_spans(spans, 10)
    assert length(wrapped) == 2
    assert Enum.map(Enum.at(wrapped, 0), & &1.content) == ["1234567890"]
    assert Enum.map(Enum.at(wrapped, 1), & &1.content) == ["abcdefghij"]
  end
end
