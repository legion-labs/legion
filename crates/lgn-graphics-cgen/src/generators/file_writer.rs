#[derive(Debug)]
struct Line {
    indent: u32,
    content: Option<String>,
}

pub struct FileWriter {
    lines: Vec<Line>,
    indent: u32,
}

impl FileWriter {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            indent: 0,
        }
    }

    pub fn new_line(&mut self) {
        self.lines.push(Line {
            indent: self.indent,
            content: None,
        });
    }

    pub fn add_line<S: Into<String>>(&mut self, line: S) {
        self.lines.push(Line {
            indent: self.indent,
            content: Some(line.into()),
        });
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn unindent(&mut self) {
        assert!(self.indent > 0);
        self.indent -= 1;
    }

    pub fn build(self) -> String {
        assert_eq!(self.indent, 0);

        let mut result = String::new();

        for line in &self.lines {
            if let Some(line_content) = &line.content {
                for _ in 0..line.indent {
                    // indentation is 4 spaces
                    result.push_str("    ");
                }
                result.push_str(line_content);
            }
            result.push('\n');
        }

        result
    }
}
