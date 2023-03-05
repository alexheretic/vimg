mod command;
mod process;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
enum Command {
    Create(command::Create),
    Join(command::Join),
    Extract(command::Extract),
}

fn main() -> anyhow::Result<()> {
    let action = Command::parse();

    match action {
        Command::Create(c) => c.run()?,
        Command::Join(c) => c.run()?,
        Command::Extract(c) => _ = c.run()?,
    }

    Ok(())
}
