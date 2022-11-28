#![warn(missing_docs)]

use std::process::{Command, Stdio};


#[derive(Debug)]
pub struct FfmpegBuilder<'a> {
    pub options: Vec<Parameter<'a>>,
    pub inputs: Vec<File<'a>>,
    pub outputs: Vec<File<'a>>,

    pub ffmpeg_command: &'a str,
    pub stdin: Stdio,
    pub stdout: Stdio,
    pub stderr: Stdio,
}

#[derive(Debug)]
pub struct File<'a> {
    pub url: &'a str,
    pub options: Vec<Parameter<'a>>,
}

#[derive(Debug)]
pub enum Parameter<'a> {
    Single(&'a str),
    KeyValue(&'a str, &'a str),
}

impl<'a> FfmpegBuilder<'a> {
    pub fn new() -> FfmpegBuilder<'a> {
        FfmpegBuilder {
            options: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            ffmpeg_command: "ffmpeg",
            stdin: Stdio::null(),
            stdout: Stdio::null(),
            stderr: Stdio::null(),
        }
    }

    pub fn option(mut self, option: Parameter<'a>) -> Self {
        self.options.push(option);

        self
    }

    pub fn input(mut self, input: File<'a>) -> Self {
        self.inputs.push(input);

        self
    }

    pub fn output(mut self, output: File<'a>) -> Self {
        self.outputs.push(output);

        self
    }

    pub fn stdin(mut self, stdin: Stdio) -> Self {
        self.stdin = stdin;

        self
    }

    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = stdout;

        self
    }

    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = stderr;

        self
    }

    pub fn to_command(self) -> Command {
        let mut command = Command::new(self.ffmpeg_command);

        for option in self.options {
            option.push_to(&mut command);
        }
        for input in self.inputs {
            input.push_to(&mut command, true);
        }
        for output in self.outputs {
            output.push_to(&mut command, false)
        }

        command.stdin(self.stdin);
        command.stdout(self.stdout);
        command.stderr(self.stderr);

        command
    }
}

impl<'a> File<'a> {
    pub fn new(url: &'a str) -> File {
        File {
            url,
            options: Vec::new(),
        }
    }

    pub fn option(mut self, option: Parameter<'a>) -> Self {
        self.options.push(option);

        self
    }

    fn push_to(&self, command: &mut Command, input: bool) {
        for option in &self.options {
            option.push_to(command);
        }

        if input {
            command.arg("-i");
        }
        command.arg(&self.url);
    }
}

impl<'a> Parameter<'a> {
    fn push_to(&self, command: &mut Command) {
        match &self {
            Parameter::Single(arg) => command.arg("-".to_owned() + arg),
            Parameter::KeyValue(key, value) => {
                command.arg("-".to_owned() + key);
                command.arg(value)
            }
        };
    }
}