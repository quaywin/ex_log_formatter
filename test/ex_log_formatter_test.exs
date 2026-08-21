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
    {spans, is_error, level} = ExLogFormatter.format_line_with_meta(json_log)
    assert is_error == false
    assert level == 2
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
    {spans, is_error, level} = ExLogFormatter.format_line_with_meta(logfmt)
    assert is_error == false
    assert level == 2
    assert length(spans) > 0
    contents = Enum.map(spans, & &1.content)
    assert "time=" in contents
    assert "[INFO]" in contents
  end

  test "parses traditional bracket level text logs" do
    text = "2026-08-04T13:14:00Z [ERROR] Database connection refused on port 5432"
    {spans, is_error, level} = ExLogFormatter.format_line_with_meta(text)
    assert is_error == true
    assert level == 4
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
    {spans, _, _} = ExLogFormatter.format_line_with_meta("1234567890abcdefghij")
    wrapped = ExLogFormatter.wrap_spans(spans, 10)
    assert length(wrapped) == 2
    assert Enum.map(Enum.at(wrapped, 0), & &1.content) == ["1234567890"]
    assert Enum.map(Enum.at(wrapped, 1), & &1.content) == ["abcdefghij"]
  end

  test "accurately handles Nginx access logs with complex URL encodings without false positives" do
    line = ~S"""
    172.22.0.3 - - [20/Aug/2026:23:32:48 +0000] "GET /index.php?graphql&query=query%20GetTruyenBySlug(%24slug%3A%20ID!)%20%7B%0A%20%20id%0A%7D HTTP/1.1" 200 27851 "-" "node"
    """ |> String.trim()
    {spans, is_error, _level} = ExLogFormatter.format_line_with_meta(line)

    assert is_error == false
    contents = Enum.map(spans, & &1.content)

    # 7B should NOT be a separate token
    refute "7B" in contents
    # Method should be blue
    assert "GET" in contents
    # Status should be green
    assert "200" in contents
    # IP should be magenta
    assert "172.22.0.3" in contents
  end

  test "detects multi-language stack traces as error" do
    python_trace = "Traceback (most recent call last):\n  File \"app.py\", line 42, in <module>"
    java_trace = "Exception in thread \"main\" java.lang.NullPointerException\n\tat com.example.App.main(App.java:10)"
    go_panic = "panic: runtime error: invalid memory address or nil pointer dereference"
    elixir_err = "** (RuntimeError) something went wrong"

    assert elem(ExLogFormatter.format_line_with_meta(python_trace), 1) == true
    assert elem(ExLogFormatter.format_line_with_meta(java_trace), 1) == true
    assert elem(ExLogFormatter.format_line_with_meta(go_panic), 1) == true
    assert elem(ExLogFormatter.format_line_with_meta(elixir_err), 1) == true
  end
end
