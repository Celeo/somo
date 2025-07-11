use clap::{Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use inquire::InquireError;
use inquire::Select;
use nix::sys::signal;
use nix::unistd::Pid;
use std::{io, string::String};

use crate::schemas::Connection;
use crate::utils;

/// Used for parsing all the flag values provided by the user in the CLI.
#[derive(Debug)]
pub struct Flags {
    pub kill: bool,
    pub proto: Option<String>,
    pub ip: Option<String>,
    pub remote_port: Option<String>,
    pub port: Option<String>,
    pub program: Option<String>,
    pub pid: Option<String>,
    pub format: Option<String>,
    pub json: bool,
    pub open: bool,
    pub listen: bool,
    pub exclude_ipv6: bool,
}

/// Represents all possible flags which can be provided by the user in the CLI.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Display an interactive selection option after inspecting connections
    #[arg(short = 'k', long, default_value = None)]
    kill: bool,

    /// Filter connections by protocol, e.g., "tcp", "udp"
    #[arg(long, default_value = None)]
    proto: Option<String>,

    /// Filter connections by remote IP address
    #[arg(long, default_value = None)]
    ip: Option<String>,

    /// Filter connections by remote port
    #[arg(long, default_value = None)]
    remote_port: Option<String>,

    /// Filter connections by local port
    #[arg(short = 'p', long, default_value = None)]
    port: Option<String>,

    /// Filter connections by program name
    #[arg(long, default_value = None)]
    program: Option<String>,

    /// Filter connections by PID
    #[arg(long, default_value = None)]
    pid: Option<String>,

    /// Format the output in a certain way, e.g., `somo --format "PID: {{pid}}, Protocol: {{proto}}, Remote Address: {{remote_address}}"`
    #[arg(long, default_value = None)]
    format: Option<String>,

    /// Output in JSON
    #[arg(long, default_value_t = false)]
    json: bool,

    /// Filter by open connections
    #[arg(short = 'o', long, default_value_t = false)]
    open: bool,

    /// Filter by listening connections
    #[arg(short = 'l', long, default_value_t = false)]
    listen: bool,

    /// Exclude IPv6 connections
    #[arg(long, default_value_t = false)]
    exclude_ipv6: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate shell completions
    GenerateCompletions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub enum CliCommand {
    Run(Flags),
    Subcommand(Commands),
}

/// Gets all flag values provided by the user in the CLI using the "clap" crate.
///
/// # Arguments
/// None
///
/// # Returns
/// A `CliCommand` enum which contains either the `Run` variant with the parsed flags or the `Subcommand` variant with a specific command.
pub fn cli() -> CliCommand {
    let args = Args::parse();

    match args.command {
        Some(cmd) => CliCommand::Subcommand(cmd),
        None => CliCommand::Run(Flags {
            kill: args.kill,
            proto: args.proto,
            ip: args.ip,
            remote_port: args.remote_port,
            port: args.port,
            program: args.program,
            pid: args.pid,
            format: args.format,
            json: args.json,
            open: args.open,
            listen: args.listen,
            exclude_ipv6: args.exclude_ipv6,
        }),
    }
}

/// Generates and prints shell completions to stdout.
///
/// # Arguments
/// * `gen` - The shell to generate completions for
/// * `cmd` - The clap command to generate completions for
///
/// # Returns
/// None
pub fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

/// Kills a process by its PID.
///
/// # Argument
/// * `pid`: The PID value as a string.
///
/// # Returns
/// None
pub fn kill_process(pid_num: i32) {
    let pid = Pid::from_raw(pid_num);

    match signal::kill(pid, signal::Signal::SIGTERM) {
        Ok(_) => utils::pretty_print_info(&format!("Killed process with PID {}.", pid)),
        Err(_) => utils::pretty_print_error(&format!("Failed to kill process with PID {}.", pid)),
    }
}

/// Starts an interactive selection process in the console for choosing a process to kill using the "inquire" crate.
///
/// # Argument
/// * `connections`: A vector containing all connections which themselves contain a PID value.
///
/// # Returns
/// None
pub fn interactive_process_kill(connections: &[Connection]) {
    let selection: Result<u32, InquireError> = Select::new(
        "Which process to kill (search or type index)?",
        (1..=connections.len() as u32).collect(),
    )
    .prompt();

    match selection {
        Ok(choice) => {
            let pid_str = &connections[choice as usize - 1].pid;
            let pid_num = match pid_str.parse::<i32>() {
                Ok(pid) => pid,
                Err(_) => {
                    utils::pretty_print_error("Couldn't find PID.");
                    return;
                }
            };
            kill_process(pid_num)
        }
        Err(_) => {
            utils::pretty_print_error("Process selection cancelled.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, Commands};
    use clap::Parser;

    #[test]
    fn test_all_flags_parsing() {
        let args = Args::parse_from([
            "test-bin",
            "-k",
            "--proto",
            "udp",
            "--ip",
            "192.168.0.1",
            "--remote-port",
            "53",
            "-p",
            "8080",
            "--program",
            "nginx",
            "--pid",
            "1234",
            "-o",
            "-l",
            "--exclude-ipv6",
        ]);

        assert!(args.kill);
        assert_eq!(args.proto.as_deref(), Some("udp"));
        assert_eq!(args.ip.as_deref(), Some("192.168.0.1"));
        assert_eq!(args.remote_port.as_deref(), Some("53"));
        assert_eq!(args.port.as_deref(), Some("8080"));
        assert_eq!(args.program.as_deref(), Some("nginx"));
        assert_eq!(args.pid.as_deref(), Some("1234"));
        assert!(args.open);
        assert!(args.listen);
        assert!(args.exclude_ipv6);
    }

    #[test]
    fn test_default_values() {
        let args = Args::parse_from(["test-bin"]);

        assert!(!args.kill);
        assert!(args.proto.is_none());
        assert!(args.ip.is_none());
        assert!(args.remote_port.is_none());
        assert!(args.port.is_none());
        assert!(args.program.is_none());
        assert!(args.pid.is_none());
        assert!(!args.open);
        assert!(!args.listen);
        assert!(!args.exclude_ipv6);
    }

    #[test]
    fn test_flag_short_and_long_equivalence() {
        let short = Args::parse_from(["test-bin", "-k", "-p", "80", "-o", "-l"]);
        let long = Args::parse_from(["test-bin", "--kill", "--port", "80", "--open", "--listen"]);

        assert_eq!(short.kill, long.kill);
        assert_eq!(short.port, long.port);
        assert_eq!(short.open, long.open);
        assert_eq!(short.listen, long.listen);
        assert_eq!(short.exclude_ipv6, long.exclude_ipv6);
    }

    #[test]
    fn test_generate_completions_subcommand() {
        let args = Args::parse_from(["test-bin", "generate-completions", "bash"]);

        match args.command {
            Some(Commands::GenerateCompletions { shell }) => {
                assert_eq!(shell.to_string(), "bash");
            }
            _ => panic!("Expected GenerateCompletions command"),
        }
    }

    #[test]
    fn test_generate_completions_all_shells() {
        let shells = ["bash", "zsh", "fish", "elvish"];

        for shell in &shells {
            let args = Args::parse_from(["test-bin", "generate-completions", shell]);

            match args.command {
                Some(Commands::GenerateCompletions {
                    shell: parsed_shell,
                }) => {
                    assert_eq!(parsed_shell.to_string(), *shell);
                }
                _ => panic!("Expected GenerateCompletions command for {}", shell),
            }
        }
    }

    #[test]
    fn test_cli_returns_none_for_subcommands() {
        // Mock the Args parsing by directly testing the logic
        // This test ensures that when a subcommand is present, cli() returns None

        // We can't easily test the full cli() function without actually running the completion
        // generation, so we test the Args parsing logic instead
        let args = Args::parse_from(["test-bin", "generate-completions", "bash"]);

        // Verify that a subcommand is present
        assert!(args.command.is_some());

        // Verify that flags are still parsed correctly even with subcommands
        assert!(!args.kill);
        assert!(args.proto.is_none());
    }
}
