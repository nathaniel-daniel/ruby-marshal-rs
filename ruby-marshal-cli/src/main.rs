mod commands;

#[derive(Debug, argh::FromArgs)]
#[argh(description = "a cli for interacting with Ruby's Marshal format")]
struct Options {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Marshal2Json(self::commands::marshal2json::Options),
}

fn main() -> anyhow::Result<()> {
    let options: Options = argh::from_env();
    match options.subcommand {
        Subcommand::Marshal2Json(options) => self::commands::marshal2json::exec(options)?,
    }
    Ok(())
}
