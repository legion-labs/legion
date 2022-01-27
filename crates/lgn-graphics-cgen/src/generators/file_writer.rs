use std::ops::{Deref, DerefMut};

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

    pub fn add_line<S: AsRef<str>>(&mut self, line: S) {
        self.lines.push(Line {
            indent: self.indent,
            content: Some(line.as_ref().to_owned()),
        });
    }

    pub fn add_lines<S: AsRef<str>>(&mut self, lines: &[S]) {
        for line in lines {
            self.add_line(line);
        }        
    }

    pub fn add_block<'w, 'b, 'e, Sb: AsRef<str>, Se: AsRef<str>>(
        &'w mut self,
        begin: &'b [Sb],
        end: &'e [Se],
    ) -> FileWriterScope<'w, 'e, Se> {
        FileWriterScope::new(self, begin, end)
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn unindent(&mut self) {
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

pub struct FileWriterScope<'w, 'e, S: AsRef<str>> {
    file_writer: &'w mut FileWriter,
    end: &'e [S],
}

impl<'w, 'e, Se: AsRef<str>> FileWriterScope<'w, 'e, Se> {
    fn new<'b, Sb: AsRef<str>>(
        file_writer: &'w mut FileWriter,
        begin: &'b [Sb],
        end: &'e [Se],
    ) -> Self {
        for line in begin {
            file_writer.add_line(line);
        }
        file_writer.indent();
        Self { file_writer, end }
    }
}

impl<'w, 'e, S: AsRef<str>> Drop for FileWriterScope<'w, 'e, S> {
    fn drop(&mut self) {
        self.file_writer.unindent();
        for line in self.end {
            self.file_writer.add_line(line);
        }
    }
}

impl<'w, 'e, S: AsRef<str>> Deref for FileWriterScope<'w, 'e, S> {
    type Target = FileWriter;

    fn deref(&self) -> &Self::Target {
        self.file_writer
    }
}

impl<'w, 'e, S: AsRef<str>> DerefMut for FileWriterScope<'w, 'e, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file_writer
    }
}
