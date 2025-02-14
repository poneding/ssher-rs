use crate::config::load_config;
use clap::{builder::StyledStr, Command};
use clap_complete::{env::Shells, generate, CompletionCandidate, Shell};
use std::io;

pub(crate) fn print_completions(shell: Shell, cmd: &mut Command) {
    generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());

    println!();

    let name = cmd.get_name();
    Shells::builtins()
        .completer(shell.to_string().as_str())
        .unwrap()
        .write_registration("COMPLETE", name, name, name, &mut io::stdout())
        .unwrap();
}

pub(crate) fn servers_len() -> usize {
    load_config().servers.len()
}

pub(crate) fn server_completer(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let config = load_config();

    let current = current.to_str().unwrap_or_default();

    config
        .servers
        .iter()
        .filter(|s| s.name.contains(current) || s.host.contains(current))
        .map(|s| {
            let help = Some(StyledStr::from(format!(
                "[{}] {}@{}:{}",
                if s.current.unwrap_or_default() {
                    "✲"
                } else {
                    " "
                },
                s.user,
                s.host,
                s.port
            )));

            CompletionCandidate::new(s.name.clone()).help(help)
        })
        .collect()
}
