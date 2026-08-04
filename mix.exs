defmodule ExLogFormatter.MixProject do
  use Mix.Project

  @version "0.1.0"
  @github_url "https://github.com/quaywin/ex_log_formatter"

  def project do
    [
      app: :ex_log_formatter,
      version: @version,
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "High-performance Rust-accelerated log formatter, ANSI sanitizer, and sub-highlighter for Elixir and Ratatui TUI applications.",
      package: package(),
      docs: [
        main: "ExLogFormatter",
        source_url: @github_url
      ]
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.35"},
      {:ex_doc, ">= 0.0.0", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      name: "ex_log_formatter",
      licenses: ["MIT"],
      links: %{"GitHub" => @github_url},
      files: ~w(lib native Cargo.toml Cargo.lock mix.exs README.md LICENSE)
    ]
  end
end
