use anyhow::Result;

fn main() -> Result<()> {
    for line in minigraf_examples::scenarios::agentic_memory()? {
        println!("{line}");
    }

    Ok(())
}
