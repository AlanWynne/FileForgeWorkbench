//! Environment variable builder for child processes.
//!
//! Merges OS environment inheritance with `shell.env` configuration and
//! per-profile environment overlays. Supports platform-specific variable
//! expansion (`%VAR%` on Windows, `$VAR`/`${VAR}` on POSIX).

use std::collections::HashMap;

/// Builds the environment variable map for a child process.
///
/// Merges three sources in order of precedence (highest last):
/// 1. Inherited OS environment (base)
/// 2. `shell.env` configuration table (overrides)
/// 3. Per-profile env overlay (highest priority)
#[derive(Debug, Clone)]
pub struct EnvironmentBuilder;

impl EnvironmentBuilder {
    /// Builds the complete environment for a child process.
    ///
    /// Starts with the inherited OS environment, applies `shell.env` overrides,
    /// then applies per-profile env overrides. Variable expansion is performed
    /// on the final merged values.
    pub fn build(
        shell_env: &HashMap<String, String>,
        profile_env: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut env: HashMap<String, String> = std::env::vars().collect();

        // Apply shell.env overrides
        for (key, value) in shell_env {
            let expanded = Self::expand_variables(value, &env);
            env.insert(key.clone(), expanded);
        }

        // Apply profile env overrides (highest priority)
        for (key, value) in profile_env {
            let expanded = Self::expand_variables(value, &env);
            env.insert(key.clone(), expanded);
        }

        env
    }

    /// Builds environment from a custom base (for testing, without inheriting OS env).
    pub fn build_from_base(
        base_env: &HashMap<String, String>,
        shell_env: &HashMap<String, String>,
        profile_env: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut env = base_env.clone();

        // Apply shell.env overrides
        for (key, value) in shell_env {
            let expanded = Self::expand_variables(value, &env);
            env.insert(key.clone(), expanded);
        }

        // Apply profile env overrides (highest priority)
        for (key, value) in profile_env {
            let expanded = Self::expand_variables(value, &env);
            env.insert(key.clone(), expanded);
        }

        env
    }

    /// Expands environment variable references in a value string.
    ///
    /// On Windows: expands `%VAR%` references.
    /// On POSIX: expands `$VAR` and `${VAR}` references.
    ///
    /// Undefined variables are replaced with empty string and a DEBUG log
    /// message is emitted.
    pub fn expand_variables(value: &str, env: &HashMap<String, String>) -> String {
        #[cfg(windows)]
        {
            Self::expand_windows(value, env)
        }
        #[cfg(unix)]
        {
            Self::expand_posix(value, env)
        }
    }

    /// Expands `%VAR%` references (Windows syntax).
    #[cfg(windows)]
    fn expand_windows(value: &str, env: &HashMap<String, String>) -> String {
        let mut result = String::with_capacity(value.len());
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                // Read the variable name until the next %
                let mut var_name = String::new();
                let mut found_end = false;
                for next_ch in chars.by_ref() {
                    if next_ch == '%' {
                        found_end = true;
                        break;
                    }
                    var_name.push(next_ch);
                }
                if found_end && !var_name.is_empty() {
                    // Look up the variable
                    match env.get(&var_name) {
                        Some(val) => result.push_str(val),
                        None => {
                            ff_logging::log(
                                ff_logging::LogLevel::Debug,
                                "ff_shell::environment",
                                &format!("undefined variable referenced: {}", var_name),
                            );
                            // Replace with empty string
                        }
                    }
                } else if !found_end {
                    // Unmatched %, just output what we consumed
                    result.push('%');
                    result.push_str(&var_name);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Expands `$VAR` and `${VAR}` references (POSIX syntax).
    #[cfg(unix)]
    fn expand_posix(value: &str, env: &HashMap<String, String>) -> String {
        let mut result = String::with_capacity(value.len());
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                if chars.peek() == Some(&'{') {
                    // ${VAR} syntax
                    chars.next(); // consume '{'
                    let mut var_name = String::new();
                    let mut found_end = false;
                    for next_ch in chars.by_ref() {
                        if next_ch == '}' {
                            found_end = true;
                            break;
                        }
                        var_name.push(next_ch);
                    }
                    if found_end && !var_name.is_empty() {
                        match env.get(&var_name) {
                            Some(val) => result.push_str(val),
                            None => {
                                ff_logging::log(
                                    ff_logging::LogLevel::Debug,
                                    "ff_shell::environment",
                                    &format!("undefined variable referenced: {}", var_name),
                                );
                            }
                        }
                    } else if !found_end {
                        result.push('$');
                        result.push('{');
                        result.push_str(&var_name);
                    }
                } else {
                    // $VAR syntax (alphanumeric + underscore)
                    let mut var_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            var_name.push(next_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if !var_name.is_empty() {
                        match env.get(&var_name) {
                            Some(val) => result.push_str(val),
                            None => {
                                ff_logging::log(
                                    ff_logging::LogLevel::Debug,
                                    "ff_shell::environment",
                                    &format!("undefined variable referenced: {}", var_name),
                                );
                            }
                        }
                    } else {
                        result.push('$');
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 12.1
    #[test]
    fn build_from_base_inherits_all_base_env_keys() {
        let base: HashMap<String, String> = HashMap::from([
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("HOME".to_string(), "/home/user".to_string()),
        ]);
        let shell_env = HashMap::new();
        let profile_env = HashMap::new();

        let result = EnvironmentBuilder::build_from_base(&base, &shell_env, &profile_env);

        assert_eq!(result.get("PATH"), Some(&"/usr/bin".to_string()));
        assert_eq!(result.get("HOME"), Some(&"/home/user".to_string()));
    }

    // Validates: Requirement 12.2
    #[test]
    fn shell_env_adds_new_variables() {
        let base: HashMap<String, String> =
            HashMap::from([("PATH".to_string(), "/usr/bin".to_string())]);
        let shell_env = HashMap::from([("MY_VAR".to_string(), "my_value".to_string())]);
        let profile_env = HashMap::new();

        let result = EnvironmentBuilder::build_from_base(&base, &shell_env, &profile_env);

        assert_eq!(result.get("PATH"), Some(&"/usr/bin".to_string()));
        assert_eq!(result.get("MY_VAR"), Some(&"my_value".to_string()));
    }

    // Validates: Requirement 12.3
    #[test]
    fn shell_env_overrides_on_collision() {
        let base: HashMap<String, String> =
            HashMap::from([("PATH".to_string(), "/usr/bin".to_string())]);
        let shell_env = HashMap::from([("PATH".to_string(), "/custom/bin".to_string())]);
        let profile_env = HashMap::new();

        let result = EnvironmentBuilder::build_from_base(&base, &shell_env, &profile_env);

        assert_eq!(result.get("PATH"), Some(&"/custom/bin".to_string()));
    }

    // Validates: Requirement 12.4
    #[cfg(unix)]
    #[test]
    fn posix_variable_expansion_dollar_var() {
        let env = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
        let result = EnvironmentBuilder::expand_variables("$HOME/bin", &env);
        assert_eq!(result, "/home/user/bin");
    }

    // Validates: Requirement 12.4
    #[cfg(unix)]
    #[test]
    fn posix_variable_expansion_curly_brace() {
        let env = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
        let result = EnvironmentBuilder::expand_variables("${HOME}/bin", &env);
        assert_eq!(result, "/home/user/bin");
    }

    // Validates: Requirement 12.4
    #[cfg(windows)]
    #[test]
    fn windows_variable_expansion_percent() {
        let env = HashMap::from([("USERPROFILE".to_string(), "C:\\Users\\test".to_string())]);
        let result = EnvironmentBuilder::expand_variables("%USERPROFILE%\\bin", &env);
        assert_eq!(result, "C:\\Users\\test\\bin");
    }

    // Validates: Requirement 12.5
    #[cfg(unix)]
    #[test]
    fn undefined_variable_expands_to_empty_string() {
        let env = HashMap::new();
        let result = EnvironmentBuilder::expand_variables("$UNDEFINED_VAR", &env);
        assert_eq!(result, "");
    }

    // Validates: Requirement 12.5
    #[cfg(windows)]
    #[test]
    fn undefined_variable_expands_to_empty_string() {
        let env = HashMap::new();
        let result = EnvironmentBuilder::expand_variables("%UNDEFINED_VAR%", &env);
        assert_eq!(result, "");
    }

    // Validates: Requirement 12.4
    #[cfg(unix)]
    #[test]
    fn literal_strings_without_variables_unchanged() {
        let env = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
        let result = EnvironmentBuilder::expand_variables("no variables here", &env);
        assert_eq!(result, "no variables here");
    }

    // Validates: Requirement 12.4
    #[cfg(windows)]
    #[test]
    fn literal_strings_without_variables_unchanged() {
        let env = HashMap::from([("HOME".to_string(), "C:\\Users\\test".to_string())]);
        let result = EnvironmentBuilder::expand_variables("no variables here", &env);
        assert_eq!(result, "no variables here");
    }

    // Validates: Requirement 12.2, 12.3
    #[test]
    fn profile_env_has_highest_priority() {
        let base: HashMap<String, String> =
            HashMap::from([("VAR".to_string(), "base".to_string())]);
        let shell_env = HashMap::from([("VAR".to_string(), "shell".to_string())]);
        let profile_env = HashMap::from([("VAR".to_string(), "profile".to_string())]);

        let result = EnvironmentBuilder::build_from_base(&base, &shell_env, &profile_env);

        assert_eq!(result.get("VAR"), Some(&"profile".to_string()));
    }
}
