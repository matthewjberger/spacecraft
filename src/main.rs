use nightshade::prelude::*;
use template_core::Spacecraft;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Spacecraft::default())?;
    Ok(())
}
