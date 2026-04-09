
fn format_toml_error(content: &str, error: &toml::de::Error, file_path: Option<&str>) -> String {
    let mut output = String::new();

    // Header with file info
    if let Some(path) = file_path {
        writeln!(output, "┌─ Error in {}", path).unwrap();
    } else {
        writeln!(output, "┌─ TOML Parse Error").unwrap();
    }

    writeln!(output, "│").unwrap();

    // Get error details
    let message = error.message();
    let span = error.span();

    if let Some(span) = span {
        let lines: Vec<&str> = content.lines().collect();
        let start_line = content[..span.start].lines().count();
        let end_line = content[..span.end].lines().count();
        let start_col = content[..span.start].lines().last().map_or(0, |l| l.len());
        let end_col = if start_line == end_line {
            start_col + (span.end - span.start)
        } else {
            content[..span.end].lines().last().map_or(0, |l| l.len())
        };

        // Show error message
        writeln!(output, "│ ❌ {}", message).unwrap();
        writeln!(output, "│").unwrap();

        // Show line numbers and context
        let context_start = start_line.saturating_sub(2);
        let context_end = (end_line + 2).min(lines.len());

        for (i, line) in lines
            .iter()
            .enumerate()
            .take(context_end)
            .skip(context_start)
        {
            let line_num = i + 1;
            let is_error_line = line_num + 1 > start_line && line_num + 1 <= end_line + 1;

            if is_error_line {
                // Error line with highlighting
                writeln!(output, "│ {:3} │ {}", line_num, line).unwrap();

                // Add error pointer
                if line_num + 1 == start_line + 1 {
                    let pointer_start = start_col;
                    let pointer_len = if start_line == end_line {
                        (end_col - start_col).max(1)
                    } else {
                        line.len() - start_col
                    };

                    write!(output, "│     │ ").unwrap();
                    write!(output, "{}", " ".repeat(pointer_start)).unwrap();
                    write!(output, "{}", "^".repeat(pointer_len.max(1))).unwrap();
                    writeln!(output, " {}", message).unwrap();
                }
            } else {
                // Context line
                writeln!(output, "│ {:3} │ {}", line_num, line).unwrap();
            }
        }

        writeln!(output, "│").unwrap();
        writeln!(
            output,
            "└─ at line {}, column {}",
            start_line + 1,
            start_col + 1
        )
        .unwrap();
    } else {
        // No span information available
        writeln!(output, "│ ❌ {}", message).unwrap();
        writeln!(output, "│").unwrap();
        writeln!(output, "└─ Unable to determine exact location").unwrap();
    }

    output
}
