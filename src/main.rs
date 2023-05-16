mod command;
mod process;
mod temporary;

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
    let cmd = Command::parse();

    _ = ctrlc::set_handler(|| {
        temporary::clean();
        std::process::exit(1);
    });

    let result = run(cmd);

    temporary::clean();

    result
}

fn run(cmd: Command) -> anyhow::Result<()> {
    match cmd {
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
