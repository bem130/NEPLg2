use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use nepl_core::{
    compile_module, loader::Loader, CompilationArtifact, CompileOptions, CompileTarget,
};
use wasmi::{Caller, Engine, Linker, Module, Store};

/// コマンドライン引数を定義するための構造体
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(
        long,
        value_name = "FORMAT",
        default_value = "wasm",
        help = "Output format: wasm"
    )]
    emit: String,

    #[arg(long, help = "Run the code if the output format is wasm")]
    run: bool,
    #[arg(
        long,
        help = "Compile as library (do not wrap top-level in an implicit main)"
    )]
    lib: bool,

    #[arg(long, value_name = "TARGET", default_value = "wasm", value_parser = ["wasm", "wasi"], help = "Compilation target: wasm or wasi")]
    target: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    if !cli.run && cli.output.is_none() {
        return Err(anyhow::anyhow!("Either --run or --output is required"));
    }
    let (module, _sm) = match cli.input {
        Some(path) => {
            let loader = Loader::new(stdlib_root()?);
            loader
                .load(&PathBuf::from(path))
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            let loader = Loader::new(stdlib_root()?);
            loader
                .load_inline(PathBuf::from("<stdin>"), buffer)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        }
    };

    let target = match cli.target.as_str() {
        "wasi" => CompileTarget::Wasi,
        _ => CompileTarget::Wasm,
    };
    let options = CompileOptions { target };

    match cli.emit.as_str() {
        "wasm" => {
            let artifact = compile_module(module, options).map_err(|e| anyhow::anyhow!("{e:?}"))?;
            if let Some(out) = &cli.output {
                write_output(out, &artifact.wasm)?;
            }
            if cli.run {
                if matches!(target, CompileTarget::Wasi) {
                    return Err(anyhow::anyhow!("--run with target=wasi is not supported in embedded runner; use a WASI runtime (e.g., wasmtime)"));
                }
                let result = run_wasm(&artifact)?;
                println!("Program exited with {result}");
            }
        }
        other => return Err(anyhow::anyhow!("unsupported emit format: {other}")),
    }

    if cli.lib {
        eprintln!("--lib is acknowledged but not yet implemented in the placeholder pipeline");
    }

    Ok(())
}

fn write_output(path: &str, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = PathBuf::from(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {parent:?}"))?;
        }
    }
    fs::write(path, bytes).with_context(|| format!("failed to write output file {path}"))?;
    Ok(())
}

fn run_wasm(artifact: &CompilationArtifact) -> Result<i32> {
    let engine = Engine::default();
    let module = Module::new(&engine, artifact.wasm.as_slice())
        .context("failed to compile wasm artifact")?;
    let mut linker = Linker::new(&engine);
    linker.func_wrap("env", "print_i32", |x: i32| {
        println!("{x}");
    })?;
    linker.func_wrap(
        "env",
        "print_str",
        |mut caller: Caller<'_, ()>, ptr: i32| {
            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export not found");
            let data = memory.data(&caller);
            let offset = ptr as usize;
            if offset + 4 > data.len() {
                panic!("print_str: pointer out of bounds");
            }
            let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            let start = offset + 4;
            if start + len > data.len() {
                panic!("print_str: slice out of bounds");
            }
            let bytes = &data[start..start + len];
            let text = std::str::from_utf8(bytes).unwrap_or("<invalid utf8>");
            print!("{text}");
        },
    )?;
    let mut store = Store::new(&engine, ());
    let instance_pre = linker
        .instantiate(&mut store, &module)
        .context("failed to instantiate module")?;
    let instance = instance_pre
        .start(&mut store)
        .context("failed to start module")?;
    if let Ok(main) = instance.get_typed_func::<(), i32>(&store, "main") {
        main.call(&mut store, ()).context("failed to execute main")
    } else if let Ok(main_unit) = instance.get_typed_func::<(), ()>(&store, "main") {
        main_unit
            .call(&mut store, ())
            .context("failed to execute main")?;
        Ok(0)
    } else {
        Err(anyhow::anyhow!(
            "exported main function missing or has wrong type"
        ))
    }
}

fn stdlib_root() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["nepl-cli", "--run"]);
        assert_eq!(cli.emit, "wasm");
        assert!(cli.run);
        assert!(cli.output.is_none());
    }
}
