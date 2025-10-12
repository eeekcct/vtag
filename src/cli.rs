use crate::git;
use crate::tag;

pub fn run(tag: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let branch = git::get_current_branch()?;
    if branch != "main" {
        return Err(Box::from("Not on 'main' branch"));
    }

    if !git::is_clean_working_tree()? {
        return Err(Box::from("Working tree is not clean"));
    }

    if !git::is_fetch_and_check_clean()? {
        return Err(Box::from("Local branch is behind remote"));
    }

    let new_tag = if let Some(tag) = tag {
        tag.to_string()
    } else {
        let bump = tag::select_bump_type()?;
        let latest =
            git::get_latest_version_tag().unwrap_or_else(|_| semver::Version::new(0, 0, 0));
        let new_version = tag::bump_version(&latest, bump);
        new_version.to_string()
    };

    if !tag::check_create_tag(&new_tag, &branch) {
        println!("🚫 Tag creation cancelled");
        return Ok(());
    }

    git::create_tag(&new_tag)?;
    git::push_tags(&new_tag)?;
    println!("🚀 Creating and pushing tag '{}'", new_tag);

    Ok(())
}
