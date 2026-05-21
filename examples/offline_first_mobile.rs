use anyhow::Result;

fn main() -> Result<()> {
    for line in minigraf_examples::scenarios::offline_first_mobile()? {
        println!("{line}");
    }

    Ok(())
}
