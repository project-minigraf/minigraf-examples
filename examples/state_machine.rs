use anyhow::Result;

fn main() -> Result<()> {
    for line in minigraf_examples::scenarios::state_machine()? {
        println!("{line}");
    }

    Ok(())
}
