use crate::git;

pub fn run(tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let branch = git::get_current_branch()?;
    println!("Creating and pushing tag: {} on branch: {}", tag, branch);
    Ok(())
}
