# Changelog

## 0.2.0 (2026-04-27)

- Add `Config::get_array(key)` typed getter returning `Option<&[String]>` for `ConfigValue::Array` values
- Add `ConfigBuilder::add_file_optional(path)` that silently skips missing files (parse errors still surface)

## 0.1.10 (2026-03-31)

- Standardize README to 3-badge format with emoji Support section
- Update CI checkout action to v5 for Node.js 24 compatibility

## 0.1.9 (2026-03-27)

- Add GitHub issue templates, PR template, and dependabot configuration
- Update README badges and add Support section

## 0.1.8 (2026-03-22)

- Fix CHANGELOG compliance

## 0.1.7 (2026-03-17)

- Add readme, rust-version, documentation to Cargo.toml
- Add Development section to README

## 0.1.6 (2026-03-16)

- Update install snippet to use full version

## 0.1.5 (2026-03-16)

- Add README badges
- Synchronize version across Cargo.toml, README, and CHANGELOG

## 0.1.0 (2026-03-15)

- Initial release
- Layered configuration: defaults, files, environment variables, manual overrides
- Built-in TOML-subset parser (strings, integers, floats, booleans, string arrays, sections)
- Environment variable mapping with prefix and double-underscore nesting
- Zero dependencies
