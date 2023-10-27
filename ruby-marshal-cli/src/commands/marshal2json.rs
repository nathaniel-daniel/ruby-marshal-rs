use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use base64::Engine;
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

    #[argh(
        switch,
        long = "convert-binary-strings-to-base64",
        description = "convert binary strings to base64"
    )]
    pub convert_binary_strings_to_base64: bool,
}

struct ConvertOptions {
    convert_binary_strings_to_base64: bool,
}

fn ruby2json_value(
    arena: &ruby_marshal::ValueArena,
    handle: ruby_marshal::ValueHandle,
    options: &ConvertOptions,
) -> anyhow::Result<serde_json::Value> {
    let value = arena.get(handle).context("missing handle")?;
    match value {
        ruby_marshal::Value::Nil(_) => Ok(serde_json::Value::Null),
        ruby_marshal::Value::False(_) => Ok(serde_json::Value::Bool(false)),
        ruby_marshal::Value::True(_) => Ok(serde_json::Value::Bool(true)),
        ruby_marshal::Value::Symbol(_value) => bail!("cannot convert a Symbol to Json"),
        ruby_marshal::Value::Fixnum(value) => Ok(serde_json::Value::Number(value.value().into())),
        ruby_marshal::Value::Array(value) => {
            let value = value.value();

            let mut array = Vec::with_capacity(value.len());
            for handle in value {
                array.push(ruby2json_value(arena, *handle, options)?);
            }

            Ok(serde_json::Value::Array(array))
        }
        ruby_marshal::Value::String(value) => {
            let instance_variables = value.instance_variables();
            let encoding = instance_variables.and_then(|instance_variables| {
                instance_variables.iter().find_map(|(key, value)| {
                    let name = arena.get_symbol(*key)?.value();
                    let value = arena.get(*value)?;

                    if name == b"encoding" || name == b"E" {
                        Some(value)
                    } else {
                        None
                    }
                })
            });

            match encoding {
                Some(_encoding) => {
                    bail!("cannot convert a String to Json")
                }
                None => {
                    ensure!(options.convert_binary_strings_to_base64, "cannot convert a binary String to Json. Consider using the \"--convert-binary-strings-to-base64\" switch.");

                    Ok(serde_json::Value::String(
                        base64::engine::general_purpose::STANDARD.encode(value.value()),
                    ))
                }
            }
        }
    }
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let file = std::fs::read(&options.input)
        .with_context(|| format!("failed to read file at \"{}\"", options.input.display()))?;
    let value_arena = ruby_marshal::load(&*file)
        .with_context(|| format!("failed to parse file at \"{}\"", options.input.display()))?;

    let json_value = ruby2json_value(
        &value_arena,
        value_arena.root(),
        &ConvertOptions {
            convert_binary_strings_to_base64: options.convert_binary_strings_to_base64,
        },
    )
    .context("failed to convert to json")?;
    let output_data = serde_json::to_string(&json_value)?;

    let output_tmp = nd_util::with_push_extension(&options.output, "tmp");
    std::fs::write(&output_tmp, output_data.as_bytes())?;
    std::fs::rename(&output_tmp, &options.output)?;

    Ok(())
}
