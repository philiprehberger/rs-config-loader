//! # philiprehberger-config-loader
//!
//! Layered configuration from files and environment variables with zero dependencies.
//!
//! Configuration is assembled from multiple sources with clear priority ordering:
//! defaults < file values < environment variables < manual overrides.
//!
//! # Example
//!
//! ```rust
//! use philiprehberger_config_loader::{ConfigBuilder, ConfigValue};
//!
//! let config = ConfigBuilder::new()
//!     .default("host", "localhost")
//!     .default("port", 8080_i64)
//!     .default("debug", false)
//!     .set("version", "1.0.0")
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(config.get_string("host"), Some("localhost"));
//! assert_eq!(config.get_int("port"), Some(8080));
//! assert_eq!(config.get_bool("debug"), Some(false));
//! ```

use std::collections::HashMap;
use std::fmt;
use std::fs;

/// A configuration value that can hold different types.
#[derive(Clone, Debug, PartialEq)]
pub enum ConfigValue {
    /// A string value.
    String(String),
    /// A 64-bit integer value.
    Integer(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
    /// An array of strings.
    Array(Vec<String>),
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::String(s) => write!(f, "{}", s),
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(v) => write!(f, "{}", v),
            ConfigValue::Bool(b) => write!(f, "{}", b),
            ConfigValue::Array(a) => {
                write!(f, "[")?;
                for (i, s) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\"", s)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl From<&str> for ConfigValue {
    fn from(s: &str) -> Self {
        ConfigValue::String(s.to_string())
    }
}

impl From<String> for ConfigValue {
    fn from(s: String) -> Self {
        ConfigValue::String(s)
    }
}

impl From<i64> for ConfigValue {
    fn from(i: i64) -> Self {
        ConfigValue::Integer(i)
    }
}

impl From<f64> for ConfigValue {
    fn from(f: f64) -> Self {
        ConfigValue::Float(f)
    }
}

impl From<bool> for ConfigValue {
    fn from(b: bool) -> Self {
        ConfigValue::Bool(b)
    }
}

impl From<Vec<String>> for ConfigValue {
    fn from(v: Vec<String>) -> Self {
        ConfigValue::Array(v)
    }
}

/// Errors that can occur during configuration loading.
#[derive(Clone, Debug, PartialEq)]
pub enum ConfigError {
    /// The specified configuration file was not found.
    FileNotFound(String),
    /// A parse error occurred while reading a configuration file.
    ParseError {
        /// The file that contained the error.
        file: String,
        /// The line number where the error occurred (1-based).
        line: usize,
        /// A description of the error.
        message: String,
    },
    /// A type mismatch occurred when retrieving a configuration value.
    TypeError {
        /// The configuration key.
        key: String,
        /// The expected type name.
        expected: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "config file not found: {}", path),
            ConfigError::ParseError { file, line, message } => {
                write!(f, "parse error in {} at line {}: {}", file, line, message)
            }
            ConfigError::TypeError { key, expected } => {
                write!(f, "type error for key '{}': expected {}", key, expected)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// An immutable configuration store.
///
/// Created via [`ConfigBuilder::build`]. Provides typed getters for
/// retrieving configuration values by key.
#[derive(Debug)]
pub struct Config {
    values: HashMap<String, ConfigValue>,
}

impl Config {
    /// Get a configuration value by key.
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }

    /// Get a string configuration value by key.
    ///
    /// Returns `None` if the key does not exist or is not a string.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(ConfigValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get an integer configuration value by key.
    ///
    /// Returns `None` if the key does not exist or is not an integer.
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.values.get(key) {
            Some(ConfigValue::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    /// Get a float configuration value by key.
    ///
    /// Returns `None` if the key does not exist or is not a float.
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.values.get(key) {
            Some(ConfigValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// Get a boolean configuration value by key.
    ///
    /// Returns `None` if the key does not exist or is not a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.values.get(key) {
            Some(ConfigValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Iterate over all configuration keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

/// Source of configuration values, applied in order during build.
enum ConfigSource {
    /// A TOML file to be parsed at build time.
    File(String),
    /// Environment variables with the given prefix.
    EnvPrefix(String),
}

/// Builder for assembling configuration from multiple layered sources.
///
/// Sources are applied in the following priority order (later overrides earlier):
/// 1. Defaults
/// 2. File values
/// 3. Environment variables
/// 4. Manual overrides
///
/// # Example
///
/// ```rust
/// use philiprehberger_config_loader::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .default("host", "0.0.0.0")
///     .default("port", 3000_i64)
///     .build()
///     .unwrap();
///
/// assert_eq!(config.get_string("host"), Some("0.0.0.0"));
/// ```
pub struct ConfigBuilder {
    defaults: HashMap<String, ConfigValue>,
    sources: Vec<ConfigSource>,
    overrides: HashMap<String, ConfigValue>,
}

impl ConfigBuilder {
    /// Create a new, empty configuration builder.
    pub fn new() -> Self {
        ConfigBuilder {
            defaults: HashMap::new(),
            sources: Vec::new(),
            overrides: HashMap::new(),
        }
    }

    /// Set a default value for the given key.
    ///
    /// Defaults have the lowest priority and are overridden by all other sources.
    pub fn default(mut self, key: &str, value: impl Into<ConfigValue>) -> Self {
        self.defaults.insert(key.to_string(), value.into());
        self
    }

    /// Add a TOML file as a configuration source.
    ///
    /// The file is parsed when [`build`](Self::build) is called. File values
    /// override defaults but are overridden by environment variables and manual overrides.
    pub fn add_file(mut self, path: &str) -> Self {
        self.sources.push(ConfigSource::File(path.to_string()));
        self
    }

    /// Add environment variables as a configuration source.
    ///
    /// Variables matching `PREFIX_KEY` are mapped to `key` (lowercased).
    /// Double underscores map to dot-separated nesting:
    /// `PREFIX_DATABASE__URL` becomes `database.url`.
    ///
    /// Environment variables override defaults and file values, but are
    /// overridden by manual overrides.
    pub fn add_env_prefix(mut self, prefix: &str) -> Self {
        self.sources.push(ConfigSource::EnvPrefix(prefix.to_string()));
        self
    }

    /// Set a manual override for the given key.
    ///
    /// Manual overrides have the highest priority and override all other sources.
    pub fn set(mut self, key: &str, value: impl Into<ConfigValue>) -> Self {
        self.overrides.insert(key.to_string(), value.into());
        self
    }

    /// Build the configuration by applying all layers in priority order.
    ///
    /// Returns a [`ConfigError`] if a file cannot be found or parsed.
    pub fn build(self) -> Result<Config, ConfigError> {
        let mut values = HashMap::new();

        // Layer 1: defaults
        for (k, v) in self.defaults {
            values.insert(k, v);
        }

        // Layer 2 & 3: sources in order (files then env prefixes)
        for source in &self.sources {
            match source {
                ConfigSource::File(path) => {
                    let content = fs::read_to_string(path).map_err(|_| {
                        ConfigError::FileNotFound(path.clone())
                    })?;
                    let parsed = parse_toml(&content, path)?;
                    for (k, v) in parsed {
                        values.insert(k, v);
                    }
                }
                ConfigSource::EnvPrefix(prefix) => {
                    let prefix_upper = format!("{}_", prefix.to_uppercase());
                    for (raw_key, raw_val) in std::env::vars() {
                        if let Some(suffix) = raw_key.strip_prefix(&prefix_upper) {
                            let config_key = suffix.to_lowercase().replace("__", ".");
                            values.insert(config_key, ConfigValue::String(raw_val));
                        }
                    }
                }
            }
        }

        // Layer 4: manual overrides
        for (k, v) in self.overrides {
            values.insert(k, v);
        }

        Ok(Config { values })
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a TOML-subset string into a flat key-value map.
///
/// Supports:
/// - `[section]` and `[section.subsection]` headers
/// - String values: `key = "value"`
/// - Integer values: `key = 123`
/// - Float values: `key = 1.5`
/// - Boolean values: `key = true` / `key = false`
/// - String arrays: `key = ["a", "b", "c"]`
/// - Comments starting with `#`
/// - Empty lines
fn parse_toml(content: &str, file: &str) -> Result<HashMap<String, ConfigValue>, ConfigError> {
    let mut map = HashMap::new();
    let mut current_section = String::new();

    for (line_idx, raw_line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Strip comments (but not inside quoted strings for simplicity,
        // we handle the common case where # appears outside values)
        let line = strip_comment(raw_line).trim();

        if line.is_empty() {
            continue;
        }

        // Section header
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(ConfigError::ParseError {
                    file: file.to_string(),
                    line: line_num,
                    message: "unclosed section header".to_string(),
                });
            }
            current_section = line[1..line.len() - 1].trim().to_string();
            continue;
        }

        // Key = value
        let eq_pos = match line.find('=') {
            Some(pos) => pos,
            None => {
                return Err(ConfigError::ParseError {
                    file: file.to_string(),
                    line: line_num,
                    message: "expected key = value".to_string(),
                });
            }
        };

        let key = line[..eq_pos].trim();
        let val_str = line[eq_pos + 1..].trim();

        let full_key = if current_section.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", current_section, key)
        };

        let value = parse_value(val_str).map_err(|msg| ConfigError::ParseError {
            file: file.to_string(),
            line: line_num,
            message: msg,
        })?;

        map.insert(full_key, value);
    }

    Ok(map)
}

/// Strip a trailing comment from a line.
///
/// This is a simple approach: find `#` that is not inside a quoted string.
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut in_array = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_string = !in_string,
            '[' if !in_string => in_array = true,
            ']' if !in_string => in_array = false,
            '#' if !in_string && !in_array => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Parse a single TOML value string into a `ConfigValue`.
fn parse_value(s: &str) -> Result<ConfigValue, String> {
    if s.is_empty() {
        return Err("empty value".to_string());
    }

    // String value
    if s.starts_with('"') {
        if !s.ends_with('"') || s.len() < 2 {
            return Err("unterminated string".to_string());
        }
        let inner = &s[1..s.len() - 1];
        return Ok(ConfigValue::String(inner.to_string()));
    }

    // Boolean
    if s == "true" {
        return Ok(ConfigValue::Bool(true));
    }
    if s == "false" {
        return Ok(ConfigValue::Bool(false));
    }

    // Array
    if s.starts_with('[') {
        if !s.ends_with(']') {
            return Err("unterminated array".to_string());
        }
        let inner = s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Ok(ConfigValue::Array(Vec::new()));
        }
        let items = parse_string_array(inner)?;
        return Ok(ConfigValue::Array(items));
    }

    // Try integer first, then float
    if let Ok(i) = s.parse::<i64>() {
        return Ok(ConfigValue::Integer(i));
    }

    if let Ok(f) = s.parse::<f64>() {
        return Ok(ConfigValue::Float(f));
    }

    Err(format!("unrecognized value: {}", s))
}

/// Parse the inner content of a string array like `"a", "b", "c"`.
fn parse_string_array(s: &str) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let mut rest = s.trim();

    loop {
        if rest.is_empty() {
            break;
        }

        if !rest.starts_with('"') {
            return Err("expected quoted string in array".to_string());
        }

        // Find the closing quote
        let inner_start = 1;
        let closing = rest[inner_start..]
            .find('"')
            .ok_or_else(|| "unterminated string in array".to_string())?;
        let value = &rest[inner_start..inner_start + closing];
        items.push(value.to_string());

        rest = rest[inner_start + closing + 1..].trim();

        // Expect comma or end
        if rest.starts_with(',') {
            rest = rest[1..].trim();
            // Allow trailing comma
        }
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn build_with_defaults_only() {
        let config = ConfigBuilder::new()
            .default("host", "localhost")
            .default("port", 8080_i64)
            .default("debug", false)
            .build()
            .unwrap();

        assert_eq!(config.get_string("host"), Some("localhost"));
        assert_eq!(config.get_int("port"), Some(8080));
        assert_eq!(config.get_bool("debug"), Some(false));
    }

    #[test]
    fn build_with_file() {
        let dir = std::env::temp_dir().join("rs_config_loader_test_file");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("config.toml");

        {
            let mut f = fs::File::create(&file_path).unwrap();
            writeln!(f, "host = \"0.0.0.0\"").unwrap();
            writeln!(f, "port = 3000").unwrap();
            writeln!(f, "ratio = 1.5").unwrap();
            writeln!(f, "debug = true").unwrap();
        }

        let config = ConfigBuilder::new()
            .default("host", "localhost")
            .add_file(file_path.to_str().unwrap())
            .build()
            .unwrap();

        assert_eq!(config.get_string("host"), Some("0.0.0.0"));
        assert_eq!(config.get_int("port"), Some(3000));
        assert_eq!(config.get_float("ratio"), Some(1.5));
        assert_eq!(config.get_bool("debug"), Some(true));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn env_var_loading_with_prefix() {
        // Set env vars for this test
        std::env::set_var("TESTCFG_HOST", "envhost");
        std::env::set_var("TESTCFG_PORT", "9090");

        let config = ConfigBuilder::new()
            .default("host", "localhost")
            .add_env_prefix("TESTCFG")
            .build()
            .unwrap();

        assert_eq!(config.get_string("host"), Some("envhost"));
        assert_eq!(config.get_string("port"), Some("9090"));

        std::env::remove_var("TESTCFG_HOST");
        std::env::remove_var("TESTCFG_PORT");
    }

    #[test]
    fn env_var_double_underscore_nesting() {
        std::env::set_var("MYAPP_DATABASE__URL", "postgres://localhost/db");
        std::env::set_var("MYAPP_DATABASE__POOL__SIZE", "10");

        let config = ConfigBuilder::new()
            .add_env_prefix("MYAPP")
            .build()
            .unwrap();

        assert_eq!(
            config.get_string("database.url"),
            Some("postgres://localhost/db")
        );
        assert_eq!(config.get_string("database.pool.size"), Some("10"));

        std::env::remove_var("MYAPP_DATABASE__URL");
        std::env::remove_var("MYAPP_DATABASE__POOL__SIZE");
    }

    #[test]
    fn layer_priority_order() {
        let dir = std::env::temp_dir().join("rs_config_loader_test_priority");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("priority.toml");

        {
            let mut f = fs::File::create(&file_path).unwrap();
            writeln!(f, "host = \"filehost\"").unwrap();
            writeln!(f, "port = 5000").unwrap();
        }

        std::env::set_var("PRI_HOST", "envhost");

        let config = ConfigBuilder::new()
            .default("host", "defaulthost")
            .default("port", 1000_i64)
            .default("name", "myapp")
            .add_file(file_path.to_str().unwrap())
            .add_env_prefix("PRI")
            .build()
            .unwrap();

        // env overrides file which overrides default
        assert_eq!(config.get_string("host"), Some("envhost"));
        // file overrides default
        assert_eq!(config.get_int("port"), Some(5000));
        // default remains
        assert_eq!(config.get_string("name"), Some("myapp"));

        std::env::remove_var("PRI_HOST");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn manual_set_overrides_everything() {
        std::env::set_var("SETTEST_HOST", "envhost");

        let config = ConfigBuilder::new()
            .default("host", "defaulthost")
            .add_env_prefix("SETTEST")
            .set("host", "override")
            .build()
            .unwrap();

        assert_eq!(config.get_string("host"), Some("override"));

        std::env::remove_var("SETTEST_HOST");
    }

    #[test]
    fn toml_parse_strings() {
        let toml = r#"name = "hello world""#;
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(
            map.get("name"),
            Some(&ConfigValue::String("hello world".to_string()))
        );
    }

    #[test]
    fn toml_parse_integers() {
        let toml = "port = 8080\nnegative = -42";
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(map.get("port"), Some(&ConfigValue::Integer(8080)));
        assert_eq!(map.get("negative"), Some(&ConfigValue::Integer(-42)));
    }

    #[test]
    fn toml_parse_floats() {
        let toml = "ratio = 3.14\nsmall = 0.001";
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(map.get("ratio"), Some(&ConfigValue::Float(3.14)));
        assert_eq!(map.get("small"), Some(&ConfigValue::Float(0.001)));
    }

    #[test]
    fn toml_parse_booleans() {
        let toml = "enabled = true\nverbose = false";
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(map.get("enabled"), Some(&ConfigValue::Bool(true)));
        assert_eq!(map.get("verbose"), Some(&ConfigValue::Bool(false)));
    }

    #[test]
    fn toml_parse_arrays() {
        let toml = r#"tags = ["alpha", "beta", "gamma"]"#;
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(
            map.get("tags"),
            Some(&ConfigValue::Array(vec![
                "alpha".to_string(),
                "beta".to_string(),
                "gamma".to_string(),
            ]))
        );
    }

    #[test]
    fn toml_parse_empty_array() {
        let toml = "items = []";
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(map.get("items"), Some(&ConfigValue::Array(Vec::new())));
    }

    #[test]
    fn toml_parse_sections() {
        let toml = "\
[database]
host = \"localhost\"
port = 5432

[database.pool]
size = 10
";
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(
            map.get("database.host"),
            Some(&ConfigValue::String("localhost".to_string()))
        );
        assert_eq!(map.get("database.port"), Some(&ConfigValue::Integer(5432)));
        assert_eq!(
            map.get("database.pool.size"),
            Some(&ConfigValue::Integer(10))
        );
    }

    #[test]
    fn toml_parse_comments_and_blank_lines() {
        let toml = "\
# This is a comment
host = \"localhost\"

# Another comment
port = 3000
";
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("host"),
            Some(&ConfigValue::String("localhost".to_string()))
        );
        assert_eq!(map.get("port"), Some(&ConfigValue::Integer(3000)));
    }

    #[test]
    fn toml_parse_inline_comment() {
        let toml = "port = 8080 # the port";
        let map = parse_toml(toml, "test").unwrap();
        assert_eq!(map.get("port"), Some(&ConfigValue::Integer(8080)));
    }

    #[test]
    fn missing_file_error() {
        let result = ConfigBuilder::new()
            .add_file("/nonexistent/path/config.toml")
            .build();

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::FileNotFound(path) => {
                assert_eq!(path, "/nonexistent/path/config.toml");
            }
            other => panic!("expected FileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn typed_getters_return_none_on_type_mismatch() {
        let config = ConfigBuilder::new()
            .default("name", "hello")
            .default("port", 8080_i64)
            .build()
            .unwrap();

        // name is a string, not an int
        assert_eq!(config.get_int("name"), None);
        // port is an int, not a string
        assert_eq!(config.get_string("port"), None);
        // nonexistent key
        assert_eq!(config.get("missing"), None);
    }

    #[test]
    fn typed_getters_return_none_for_missing_keys() {
        let config = ConfigBuilder::new().build().unwrap();

        assert_eq!(config.get("x"), None);
        assert_eq!(config.get_string("x"), None);
        assert_eq!(config.get_int("x"), None);
        assert_eq!(config.get_float("x"), None);
        assert_eq!(config.get_bool("x"), None);
    }

    #[test]
    fn config_value_display() {
        assert_eq!(ConfigValue::String("hi".into()).to_string(), "hi");
        assert_eq!(ConfigValue::Integer(42).to_string(), "42");
        assert_eq!(ConfigValue::Float(1.5).to_string(), "1.5");
        assert_eq!(ConfigValue::Bool(true).to_string(), "true");
        assert_eq!(
            ConfigValue::Array(vec!["a".into(), "b".into()]).to_string(),
            "[\"a\", \"b\"]"
        );
    }

    #[test]
    fn config_keys_iterator() {
        let config = ConfigBuilder::new()
            .default("a", "1")
            .default("b", "2")
            .build()
            .unwrap();

        let mut keys: Vec<&String> = config.keys().collect();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn config_error_display() {
        let e1 = ConfigError::FileNotFound("foo.toml".into());
        assert_eq!(e1.to_string(), "config file not found: foo.toml");

        let e2 = ConfigError::ParseError {
            file: "bar.toml".into(),
            line: 5,
            message: "bad value".into(),
        };
        assert_eq!(e2.to_string(), "parse error in bar.toml at line 5: bad value");

        let e3 = ConfigError::TypeError {
            key: "port".into(),
            expected: "integer".into(),
        };
        assert_eq!(e3.to_string(), "type error for key 'port': expected integer");
    }

    #[test]
    fn from_impls() {
        let v: ConfigValue = "hello".into();
        assert_eq!(v, ConfigValue::String("hello".into()));

        let v: ConfigValue = String::from("world").into();
        assert_eq!(v, ConfigValue::String("world".into()));

        let v: ConfigValue = 42_i64.into();
        assert_eq!(v, ConfigValue::Integer(42));

        let v: ConfigValue = 3.14_f64.into();
        assert_eq!(v, ConfigValue::Float(3.14));

        let v: ConfigValue = true.into();
        assert_eq!(v, ConfigValue::Bool(true));

        let v: ConfigValue = vec!["a".to_string(), "b".to_string()].into();
        assert_eq!(
            v,
            ConfigValue::Array(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn empty_builder() {
        let builder = ConfigBuilder::new();
        let config = builder.build().unwrap();
        assert_eq!(config.keys().count(), 0);
    }
}
