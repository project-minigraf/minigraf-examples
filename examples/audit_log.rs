use anyhow::Result;

fn main() -> Result<()> {
    for line in minigraf_examples::scenarios::audit_log()? {
        println!("{line}");
    }

    Ok(())
}
