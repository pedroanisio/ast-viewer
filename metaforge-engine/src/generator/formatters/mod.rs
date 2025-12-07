use anyhow::Result;
use std::process::{Command, Stdio};
use std::io::Write;

/// Trait for code formatters - maintains compatibility with existing code
pub trait CodeFormatter {
    fn format(&self, code: &str) -> Result<String>;
    fn is_available(&self) -> bool;
}

/// Get a formatter for a specific language - compatibility function
pub fn get_formatter(language: &str) -> Box<dyn CodeFormatter> {
    Box::new(CompatibilityFormatter::new(language))
}

/// Compatibility wrapper that implements the old CodeFormatter trait
struct CompatibilityFormatter {
    language: String,
    formatters: LanguageFormatters,
}

impl CompatibilityFormatter {
    fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            formatters: LanguageFormatters::new(),
        }
    }
}

impl CodeFormatter for CompatibilityFormatter {
    fn format(&self, code: &str) -> Result<String> {
        self.formatters.format_code(code, &self.language)
    }
    
    fn is_available(&self) -> bool {
        // Check if external formatter is available
        match self.language.to_lowercase().as_str() {
            "rust" => Command::new("rustfmt").arg("--version").output().is_ok(),
            "python" => Command::new("black").arg("--version").output().is_ok(),
            "javascript" | "typescript" => Command::new("prettier").arg("--version").output().is_ok(),
            "go" => Command::new("gofmt").arg("-h").output().is_ok(),
            "cpp" | "c++" => Command::new("clang-format").arg("--version").output().is_ok(),
            _ => true, // Basic formatting always available
        }
    }
}

/// Language formatters for Phase 1B template completion
#[derive(Debug, Clone)]
pub struct LanguageFormatters;

impl LanguageFormatters {
    pub fn new() -> Self {
        Self
    }

    /// Format code using appropriate language formatter
    pub fn format_code(&self, code: &str, language: &str) -> Result<String> {
        match language.to_lowercase().as_str() {
            "rust" => self.format_rust(code),
            "python" => self.format_python(code),
            "javascript" | "js" => self.format_javascript(code),
            "typescript" | "ts" => self.format_typescript(code),
            "go" => self.format_go(code),
            "java" => self.format_java(code),
            "csharp" | "cs" => self.format_csharp(code),
            "cpp" | "c++" => self.format_cpp(code),
            "ruby" => self.format_ruby(code),
            "php" => self.format_php(code),
            _ => Ok(code.to_string()), // Return unformatted for unsupported languages
        }
    }

    fn format_rust(&self, code: &str) -> Result<String> {
        // Use rustfmt if available
        let mut child = match Command::new("rustfmt")
            .arg("--edition")
            .arg("2021")
            .arg("--emit")
            .arg("stdout")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Ok(self.basic_rust_format(code)),
        };

        // Write code to stdin
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(code.as_bytes());
        }

        // Get output
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Ok(self.basic_rust_format(code))
                }
            }
            Err(_) => Ok(self.basic_rust_format(code)),
        }
    }

    fn format_python(&self, code: &str) -> Result<String> {
        // Try black formatter first
        let mut child = match Command::new("black")
            .arg("--line-length")
            .arg("88")
            .arg("--quiet")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Ok(self.basic_python_format(code)),
        };

        // Write code to stdin
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(code.as_bytes());
        }

        // Get output
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Ok(self.basic_python_format(code))
                }
            }
            Err(_) => Ok(self.basic_python_format(code)),
        }
    }

    fn format_javascript(&self, code: &str) -> Result<String> {
        // Use prettier if available
        let mut child = match Command::new("prettier")
            .arg("--parser")
            .arg("babel")
            .arg("--print-width")
            .arg("80")
            .arg("--tab-width")
            .arg("2")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Ok(self.basic_js_format(code)),
        };

        // Write code to stdin
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(code.as_bytes());
        }

        // Get output
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Ok(self.basic_js_format(code))
                }
            }
            Err(_) => Ok(self.basic_js_format(code)),
        }
    }

    fn format_typescript(&self, code: &str) -> Result<String> {
        // Use prettier with TypeScript parser
        let mut child = match Command::new("prettier")
            .arg("--parser")
            .arg("typescript")
            .arg("--print-width")
            .arg("80")
            .arg("--tab-width")
            .arg("2")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Ok(self.basic_js_format(code)),
        };

        // Write code to stdin
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(code.as_bytes());
        }

        // Get output
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Ok(self.basic_js_format(code))
                }
            }
            Err(_) => Ok(self.basic_js_format(code)),
        }
    }

    fn format_go(&self, code: &str) -> Result<String> {
        // Use gofmt
        let mut child = match Command::new("gofmt")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Ok(self.basic_go_format(code)),
        };

        // Write code to stdin
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(code.as_bytes());
        }

        // Get output
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Ok(self.basic_go_format(code))
                }
            }
            Err(_) => Ok(self.basic_go_format(code)),
        }
    }

    fn format_java(&self, code: &str) -> Result<String> {
        // Google Java Format or basic formatting
        Ok(self.basic_java_format(code))
    }

    fn format_csharp(&self, code: &str) -> Result<String> {
        // dotnet format or basic formatting
        Ok(self.basic_csharp_format(code))
    }

    fn format_cpp(&self, code: &str) -> Result<String> {
        // clang-format if available
        let mut child = match Command::new("clang-format")
            .arg("--style=LLVM")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Ok(self.basic_cpp_format(code)),
        };

        // Write code to stdin
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(code.as_bytes());
        }

        // Get output
        match child.wait_with_output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Ok(self.basic_cpp_format(code))
                }
            }
            Err(_) => Ok(self.basic_cpp_format(code)),
        }
    }

    fn format_ruby(&self, code: &str) -> Result<String> {
        // RuboCop or basic formatting
        Ok(self.basic_ruby_format(code))
    }

    fn format_php(&self, code: &str) -> Result<String> {
        // PHP-CS-Fixer or basic formatting
        Ok(self.basic_php_format(code))
    }

    // Basic formatting fallbacks for when external formatters aren't available

    fn basic_rust_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_python_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if trimmed.is_empty() {
                    String::new()
                } else {
                    format!("{}", trimmed)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_js_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_go_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_java_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_csharp_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_cpp_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_ruby_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn basic_php_format(&self, code: &str) -> String {
        code.lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
