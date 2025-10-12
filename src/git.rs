use git2::Repository;

pub fn get_current_branch() -> Result<String, git2::Error> {
    let repo = Repository::discover(".")?;
    let head = repo.head()?;
    if !head.is_branch() {
        return Err(git2::Error::from_str("HEAD is not pointing to a branch"));
    }
    return Ok(head.shorthand().unwrap_or("unknown").to_string());
}
