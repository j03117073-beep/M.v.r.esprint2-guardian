use anyhow::{Context, Result};
use m_v_r_esprint1::sced_chain::{build_hash_chain, read_records_from_csv, write_chain_csv};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: cargo run --bin sced_chain -- <input.csv> [output.csv]");
        std::process::exit(2);
    }

    let input = PathBuf::from(&args[1]);
    let output = if args.len() == 3 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("chain_output.csv")
    };

    let records = read_records_from_csv(&input)
        .with_context(|| format!("failed to read records from {}", input.display()))?;

    let chain = build_hash_chain(records).context("failed to build deterministic hash chain")?;

    write_chain_csv(&output, &chain)
        .with_context(|| format!("failed to write chain output to {}", output.display()))?;

    println!("Wrote {} chained records to {}", chain.len(), output.display());
    Ok(())
}
