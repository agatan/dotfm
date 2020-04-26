use std::path::Path;

use crate::walk::Walk;

struct Status {
    entry_path: String,
    target_path: String,
    is_linked: bool,
}

pub fn do_status(path: &Path) -> Result<(), anyhow::Error> {
    let walk = Walk::new(path)?;
    let status = walk
        .map(|result| {
            result.map(|entry| Status {
                entry_path: entry.display_relative().to_string(),
                target_path: entry.display_target().to_string(),
                is_linked: entry.is_linked(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for st in status {
        println!(
            "{}\t=>\t{}\t({})",
            st.entry_path,
            st.target_path,
            if st.is_linked { "linked" } else { "not linked" }
        )
    }
    Ok(())
}
