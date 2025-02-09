use crate::{
    colord_print::{green, red},
    prompt::{confirm_prompt, rename_server_prompt, yesno_select_prompt},
    save_config, server_form_prompt, servers_select_prompt,
};
use base64::{engine::general_purpose, Engine};
use ssh2::Session;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    path::Path,
};
use tabled::{settings::Style, Table};
use termion::{async_stdin, raw::IntoRawMode};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::AsyncResolver;

const VERSION: &str = "0.1.0";

pub(crate) fn version() {
    green(format!("😸 Version: v{}", VERSION).as_str());
}

pub(crate) fn list_servers(config: &crate::model::Config) {
    if config.servers.is_empty() {
        println!("😿 No servers found");
    } else {
        let table = Table::new(&config.servers)
            .with(Style::modern_rounded())
            .to_string();

        println!("{table}")
    }
}

pub(crate) fn remove_server(config: &mut crate::model::Config) {
    if let Some(server) = servers_select_prompt(&config.servers) {
        if yesno_select_prompt("Are you sure you want to remove this server?") {
            let index = config
                .servers
                .iter()
                .position(|s| s.name == server.name)
                .unwrap();

            config.servers.remove(index);
            save_config(config);

            green(format!("😺 Server {} removed.", server.name).as_str());
        }
    }
}

pub(crate) fn add_server(config: &mut crate::model::Config) {
    if let Some(server) = server_form_prompt(config) {
        let server_name = server.name.clone();

        config.servers.push(server);
        save_config(config);

        green(format!("😺 Server {} added.", server_name).as_str());
    }
}

pub(crate) fn connect_server(config: &mut crate::model::Config) {
    if let Some(server) = servers_select_prompt(&config.servers) {
        // If the server is not marked as current, mark it as current,
        // and unmark all others.
        if server.current.is_none_or(|c| !c) {
            for s in &mut config.servers {
                if s.name == server.name {
                    s.current = Some(true);
                } else {
                    s.current = Some(false);
                }
            }
            save_config(config);
        }

        let host = if server.host.parse::<std::net::IpAddr>().is_ok() {
            server.host.clone()
        } else {
            let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
            let response = resolver.lookup_ip(server.host.as_str());
            let ip = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(response)
                .expect("DNS lookup failed")
                .iter()
                .next()
                .expect("No IP addresses returned");
            ip.to_string()
        };

        let tcp =
            TcpStream::connect(format!("{}:{}", host, server.port)).expect("Failed to connect");
        tcp.set_nodelay(true).unwrap();

        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        let mut userauth = false;
        if let Some(ref password) = server.password {
            if !password.is_empty() {
                let password = general_purpose::STANDARD.decode(password).unwrap();
                sess.userauth_password(&server.user, &String::from_utf8(password).unwrap())
                    .unwrap();
                userauth = true;
            }
        }
        if !userauth {
            if let Some(ref identity_file) = server.identity_file {
                let expanded_path = shellexpand::tilde(identity_file).into_owned();
                sess.userauth_pubkey_file(&server.user, None, Path::new(&expanded_path), None)
                    .unwrap();
            }
        }

        if sess.authenticated() {
            let mut channel = sess.channel_session().unwrap();

            channel.request_pty("xterm-256color", None, None).unwrap();
            channel
                .handle_extended_data(ssh2::ExtendedData::Merge)
                .unwrap();
            channel.shell().unwrap();

            let stdout = io::stdout();
            let mut stdout = stdout.lock().into_raw_mode().unwrap();
            let mut stdin = async_stdin();

            let mut buff_in = Vec::new();
            while !channel.eof() {
                let bytes_available = channel.read_window().available;
                if bytes_available > 0 {
                    let mut buffer = vec![0; bytes_available as usize];
                    channel.read_exact(&mut buffer).unwrap();
                    stdout.write(&buffer).unwrap();
                    stdout.flush().unwrap();
                }

                // Using async_stdin to avoid blocking, should this also respect the WriteWindow?
                stdin.read_to_end(&mut buff_in).unwrap();
                channel.write(&buff_in).unwrap();
                buff_in.clear();

                // Unsure of best practice, but this feels responsive and avoids pegging the CPU
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            channel.wait_close().unwrap();
        } else {
            red("😿 Authentication failed.");
        }
    }
}

pub(crate) fn rename_server(config: &mut crate::model::Config) {
    if let Some(server) = servers_select_prompt(&config.servers) {
        let new_name = rename_server_prompt(config, &server);
        if server.name != new_name {
            for s in &mut config.servers {
                if s.name == server.name {
                    s.name = new_name.clone();
                }
            }
            save_config(config);

            green(format!("😺 Server {} renamed to {}.", server.name, new_name).as_str());
        }
    }
}
