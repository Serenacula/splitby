# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0](https://github.com/Serenacula/splitby/compare/v1.2.5...v2.0.0) - 2026-05-08

### Added

- add --terminator / -t flag
- [**breaking**] rename --skip-empty to --skip-empty-fields
- add --skip-undelimited / -s flag
- add --skip-empty-lines / -l flag
- refine config file — auto-create default, ignore unknown keys, drop delimiter/join/placeholder/strict
- add config file support at $XDG_CONFIG_HOME/splitby/config.json
- reassign -i to --invert, remove -o shorthand

### Fixed

- use splitn(2) so --flag=value with '=' in value is not truncated
- empty record counts as 0 fields; --skip-empty-lines suppresses before --count
- move --skip-empty-lines check to post-processing
- preserve batch ordering when records are skipped in transform
- replace brittle exit-code string matching with typed AppError
- whole-string mode always emits trailing newline

### Other

- add showcasing screenshots, gitignore .claude dir
- fix test suite — remove stale comments, duplicate, and add gap coverage
- simplify --no-strict description to 'disables all strict features'
- clarify --no-strict disables all strict features including the default strict-range-order
- add missing skip-empty-lines and skip-undelimited to help text
- add terminator and skip flag pages, update readme and flags index
- update planning docs with completed features
- fix inaccuracies, add long-form syntax, and improve examples
- fix inaccuracies and improve examples in readme
- put delimiter before flags in all examples
- reframe delimiter as implicit by default, -d as edge-case tool
- fix short-flag errors in readme and docs
- collapse duplicate PerLine/ZeroTerminated input branches
- remove comments that restate code, keep only non-obvious ones
- consolidate instruction structs into a single Config
- remove duplicate strict test, fix misleading and prefixed test names
