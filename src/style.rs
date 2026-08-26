use std::env;

fn enabled() -> bool {
    env::var_os("NO_COLOR").is_none()
}

fn paint(code: &str, text: &str) -> String {
    if enabled() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn green(text: &str) -> String {
    paint("32", text)
}

pub fn red(text: &str) -> String {
    paint("31", text)
}

pub fn yellow(text: &str) -> String {
    paint("33", text)
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}

pub fn dim(text: &str) -> String {
    paint("2", text)
}
