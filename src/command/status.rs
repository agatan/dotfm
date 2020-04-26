use std::path::Path;

use crate::walk::Walk;


pub fn do_status(path: &Path) -> Result<(), anyhow::Error> {
    let walk = Walk::new(path)?;
    todo!()
}
