#![allow(dead_code)]
use crate::{
    cmd,
    colord_print::yellow,
    endec,
    model::{Config, Server},
};
use dialoguer::{
    Confirm, Input, Password, Select,
    console::{Style, style},
    theme::ColorfulTheme,
};

pub(crate) fn default_theme() -> ColorfulTheme {
    ColorfulTheme {
        defaults_style: Style::new().for_stderr().cyan(),
        prompt_style: Style::new().for_stderr().bold().yellow(),
        prompt_prefix: style("❖".to_string()).for_stderr().yellow(),
        prompt_suffix: style("".to_string()).blue(),
        success_prefix: style("✔".to_string()).for_stderr().cyan(),
        success_suffix: style("".to_string()).for_stderr().black().bright(),
        error_prefix: style("✘".to_string()).for_stderr().red(),
        error_style: Style::new().for_stderr().red(),
        hint_style: Style::new().for_stderr().black().bright(),
        values_style: Style::new().for_stderr().cyan(),
        active_item_style: Style::new().for_stderr().cyan().underlined(),
        inactive_item_style: Style::new().for_stderr(),
        active_item_prefix: style("→".to_string()).for_stderr().cyan(),
        inactive_item_prefix: style(" ".to_string()).for_stderr(),
        checked_item_prefix: style("✔".to_string()).for_stderr().cyan(),
        unchecked_item_prefix: style("⬚".to_string()).for_stderr().magenta(),
        picked_item_prefix: style("→".to_string()).for_stderr().cyan(),
        unpicked_item_prefix: style(" ".to_string()).for_stderr(),
    }
}

pub(crate) fn servers_select_prompt(servers: &[Server]) -> Option<Server> {
    let max_name_width = servers.iter().map(|s| s.name.len()).max().unwrap_or(0);
    let mut selections: Vec<String> = servers
        .iter()
        .map(|s| {
            let prefix = match s.current {
                Some(true) => "✲ ",
                _ => "  ",
            };
            let subfix = match servers.len() {
                1.. if s.name == servers[servers.len() - 1].name => "\n",
                _ => "",
            };

            format!(
                "{}{:<width$}\t({}@{}:{}){}",
                prefix,
                s.name,
                s.user,
                s.host,
                s.port,
                subfix,
                width = max_name_width
            )
        })
        .collect();

    selections.push("✚ Add a new server".to_string());
    selections.push("✗ Exit".to_string());

    let current_server_index = servers.iter().position(|s| s.current == Some(true));

    let selection = Select::with_theme(&default_theme())
        .with_prompt("Select a server:")
        .default(current_server_index.unwrap_or_default())
        .report(false)
        .items(&selections)
        .interact()
        .ok()?;

    // Add a new server
    if selection == selections.len() - 2 {
        if let Err(e) = cmd::add_server() {
            yellow(format!("😾 {}", e));
        }
        return None;
    }

    // Exit
    if selection == selections.len() - 1 {
        return None;
    }

    Some(servers[selection].clone())
}

fn server_form_prompt(server: &Server, config: &Config) -> anyhow::Result<Option<Server>> {
    let name: String = Input::with_theme(&default_theme())
        .with_prompt("Name(*):")
        .with_initial_text(server.name.clone())
        .show_default(false)
        .validate_with(|input: &String| {
            if *input != server.name && config.servers.iter().any(|s| s.name == *input) {
                Err(format!("😾 Name {} already exists.", input))
            } else {
                Ok(())
            }
        })
        .allow_empty(false)
        .interact_text()?;

    let host: String = Input::with_theme(&default_theme())
        .with_prompt("Host(*):")
        .with_initial_text(server.host.clone())
        .allow_empty(false)
        .interact_text()?;

    let port: u16 = Input::with_theme(&default_theme())
        .with_prompt("Port(*):")
        .with_initial_text(server.port.to_string())
        .show_default(false)
        .allow_empty(false)
        .interact_text()?;

    let user: String = Input::with_theme(&default_theme())
        .with_prompt("User(*):")
        .default("root".to_string())
        .with_initial_text(server.user.clone())
        .show_default(false)
        .allow_empty(false)
        .interact_text()?;

    let password: String = Password::with_theme(&default_theme())
        .with_prompt("Password:")
        .allow_empty_password(true)
        .interact()?;

    let identity_file: String = Input::with_theme(&default_theme())
        .with_prompt("IdentityFile:")
        .with_initial_text(
            server
                .identity_file
                .clone()
                .unwrap_or(String::from("~/.ssh/id_rsa")),
        )
        .allow_empty(true)
        .interact_text()?;

    Ok(Some(Server {
        name,
        host,
        port,
        user,
        password: endec::encode_string(password),
        identity_file: if identity_file.is_empty() {
            None
        } else {
            Some(identity_file)
        },
        current: None,
    }))
}

pub(crate) fn add_server_form_prompt(config: &Config) -> anyhow::Result<Option<Server>> {
    let default_server = Server::new("".to_string());
    server_form_prompt(&default_server, config)
}

pub(crate) fn edit_server_form_prompt(
    config: &Config,
    server: &Server,
) -> anyhow::Result<Option<Server>> {
    server_form_prompt(server, config)
}

pub(crate) fn confirm_prompt(prompt: &str) -> anyhow::Result<bool> {
    let res = Confirm::with_theme(&default_theme())
        .with_prompt(prompt)
        .default(false)
        .report(false)
        .interact()?;

    Ok(res)
}

pub(crate) fn yesno_select_prompt(prompt: &str) -> anyhow::Result<bool> {
    let selections = vec!["No", "Yes"];
    let selection = Select::with_theme(&default_theme())
        .with_prompt(prompt)
        .default(0)
        .report(false)
        .items(&selections)
        .interact()?;

    Ok(selection == 1)
}

pub(crate) fn rename_server_prompt(config: &Config, server: &Server) -> anyhow::Result<String> {
    let res = Input::with_theme(&default_theme())
        .with_prompt("New name(*):")
        .with_initial_text(server.name.clone())
        .validate_with(|input: &String| {
            if *input == server.name {
                yellow("😺 Name not changed.");
                return Ok(());
            }

            if config.servers.iter().any(|s| s.name == *input) {
                Err(format!("😾 Name {} already exists.", input))
            } else {
                Ok(())
            }
        })
        .report(false)
        .allow_empty(false)
        .interact_text()?;

    Ok(res)
}
