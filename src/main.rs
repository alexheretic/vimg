mod command;
mod process;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
enum Command {
    Vcs(command::Vcs),
    Join(command::Join),
    Extract(command::Extract),
    PrintCompletions(command::PrintCompletions),
}

fn main() -> anyhow::Result<()> {
    let action = Command::parse();

    match action {
        Command::Vcs(c) => c.run()?,
        Command::Join(c) => c.run()?,
        Command::Extract(c) => {
            let ex = c.run()?;
            for msg in ex.warnings {
                eprintln!("Warning: {msg}");
            }
        }
        Command::PrintCompletions(c) => c.run(),
    }

    Ok(())
}
