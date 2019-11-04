use std::io;
use std::process::Command;

#[cfg(not(test))]
pub fn execute(command: &'static str) -> Option<String> {
    let (binary, arg) = split_command(command);
    Command::new(binary)
        .arg(arg)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
}

#[cfg(test)]
pub fn execute(command: &'static str) -> Option<String> {
    let (binary, arg) = split_command(command);
    let output = match binary {
        "ruby" => "ruby 2.5.5p456 (2018-03-28 revision 63024) [universal.x86_64-darwin18]",

        _ => panic!("Unknown binary"),
    };

    Some(output.to_string())
}

fn split_command(command: &'static str) -> (&'static str, &'static str) {
    let mut splitter = command.splitn(2, " ");
    let binary = splitter.next().expect("binary missing");
    let arg = splitter.next().expect("arg missing");

    (binary, arg)
}
