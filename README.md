# cata

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url] [![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/cata.svg
[crates-url]: https://crates.io/crates/cata
[docs-badge]: https://docs.rs/cata/badge.svg
[docs-url]: https://docs.rs/cata
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/grampelberg/cata/workflows/CI/badge.svg
[actions-url]: https://github.com/grampelberg/cata/actions?query=workflow%3ACI

cata(lyst) for building rust based CLIs

This crate provides a collection of utilities that make it easier to build CLI
tools. Take a look at the [docs][docs-url] for more details.

- `command`: recursively traverse a tree of clap commands and subcommands
  calling lifecycle hooks at each level.
- `file`: derive `clap::value_parser` for deserializing values from files.
  Detects the file format from the extension and currently supports JSON in
  addition to YAML.
- `output`: structured output for commands. Users can choose the output format
  they would like, currently supporting JSON, YAML and pretty.
- `telemetry`: a simple way to track activity and errors for your CLI.
