use std::io::{self, Write};

use crate::config::Config;
use crate::walk::Walk;

struct Status {
    entry_path: String,
    target_path: String,
    is_linked: bool,
}

pub fn do_status(config: &Config) -> Result<(), anyhow::Error> {
    let walk = Walk::new(config.root_path(), config.home_dir())?;
    let status = walk
        .map(|result| {
            result.map(|entry| Status {
                entry_path: entry.display_relative().to_string(),
                target_path: entry.display_target().to_string(),
                is_linked: entry.is_linked(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut writer = tabwriter::TabWriter::new(io::stdout());
    writer.write_all(b"SOURCE\tDESTINATION\tSTATUS\n")?;
    for st in status {
        writeln!(
            writer,
            "{}\t{}\t{}",
            st.entry_path,
            st.target_path,
            if st.is_linked { "linked" } else { "not linked" }
        )?;
    }
    writer.flush()?;
    Ok(())
}
