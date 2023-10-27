use anyhow::bail;
use anyhow::Context;
use std::path::PathBuf;

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "marshal2json",
    description = "turn a Ruby Marshal file into json"
)]
pub struct Options {
    #[argh(positional, description = "the input file path")]
    pub input: PathBuf,

    #[argh(positional, description = "the output file path")]
    pub output: PathBuf,
}

fn ruby2json_value(
    arena: &ruby_marshal::ValueArena,
    handle: ruby_marshal::ValueHandle,
) -> anyhow::Result<serde_json::Value> {
    let value = arena.get(handle).context("missing handle")?;
    match value {
        ruby_marshal::Value::Array(value) => {
            let value = value.value();

            let mut array = Vec::with_capacity(value.len());
            for handle in value {
                array.push(ruby2json_value(arena, *handle)?);
            }

            Ok(serde_json::Value::Array(array))
        }
        _ => bail!("unimplemented: {value:?}"),
    }
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let file = std::fs::read(&options.input)
        .with_context(|| format!("failed to read file at \"{}\"", options.input.display()))?;
    let value_arena = ruby_marshal::load(&*file)
        .with_context(|| format!("failed to parse file at \"{}\"", options.input.display()))?;

    let json_value =
        ruby2json_value(&value_arena, value_arena.root()).context("failed to convert to json")?;
    let output_data = serde_json::to_string(&json_value)?;

    let output_tmp = nd_util::with_push_extension(&options.output, "tmp");
    std::fs::write(&output_tmp, output_data.as_bytes())?;
    std::fs::rename(&output_tmp, &options.output)?;

    Ok(())
}
