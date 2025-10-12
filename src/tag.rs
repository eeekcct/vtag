use crate::git;

pub fn run(tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let branch = git::get_current_branch()?;
    println!(
        "🚀 Creating and pushing tag '{}' on branch '{}'",
        tag, branch
    );
    if branch != "main" {
        return Err(Box::from("Not on 'main' branch"));
    }

    if !git::is_clean_working_tree()? {
        return Err(Box::from("Working tree is not clean"));
    }

    if !git::is_fetch_and_check_clean()? {
        return Err(Box::from("Local branch is behind remote"));
    }

    git::create_tag(tag)?;

    Ok(())
}
