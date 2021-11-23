#[derive(Debug)]
struct Line {
    indent: u32,
    content: String,
}

pub struct FileWriter {
    lines: Vec<Line>,
    indent: u32,
}

impl FileWriter {
    pub fn new() -> Self {
        FileWriter {
            lines: Vec::new(),
            indent: 0,
        }
    }

    pub fn new_line(&mut self) {
        self.add_line("".to_owned());
    }   

    pub fn add_line(&mut self, line: String) {
        self.lines.push(Line {
            indent: self.indent,
            content: line,
        });
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn unindent(&mut self) {
        assert!(self.indent > 0);
        self.indent -= 1;
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();

        for line in &self.lines {
            for _ in 0..line.indent {
                result.push('\t');
            }
            result.push_str(&line.content);
            result.push('\n');
        }

        result
    }
}
