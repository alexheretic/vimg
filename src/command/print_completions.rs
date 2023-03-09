use clap::{CommandFactory, Parser};
use clap_complete::Shell;

/// Print shell completions.
#[derive(Parser)]
#[group(skip)]
pub struct PrintCompletions {
    /// Shell.
    #[arg(value_enum, default_value_t = Shell::Bash)]
    shell: Shell,
}

impl PrintCompletions {
    pub fn run(self) {
        clap_complete::generate(
            self.shell,
            &mut crate::Command::command(),
            "vimg",
            &mut std::io::stdout(),
        );
    }
}
