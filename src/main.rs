//! SkillSync daemon binary entrypoint

use anyhow::Result;

fn main() -> Result<()> {
    skillsync::run()
}
