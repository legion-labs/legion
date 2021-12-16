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
            for _ in 0..line.indent {
                result.push('\t');
            }
            match &line.content {
                Some(line_content) => result.push_str(line_content),
                None => (),
            }
            result.push('\n');
        }

        result
    }
}
