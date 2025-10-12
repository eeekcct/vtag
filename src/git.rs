use git2::{Error, Repository};

pub fn get_current_branch() -> Result<String, Error> {
    // Open the repository in the current directory only (no recursive search)
    let repo = Repository::open(".")?;
    let head = repo.head()?;
    if !head.is_branch() {
        return Err(Error::from_str("HEAD is not pointing to a branch"));
    }
    return Ok(head.shorthand().unwrap_or("unknown").to_string());
}

pub fn is_clean_working_tree() -> Result<bool, Error> {
    let repo = Repository::open(".")?;
    let statuses = repo.statuses(None)?;
    Ok(statuses.is_empty())
}

pub fn is_fetch_and_check_clean() -> Result<bool, Error> {
    let repo = Repository::open(".")?;
    let head = repo.head()?;
    let branch = head.shorthand().unwrap();

    // use git command to fetch
    let status = std::process::Command::new("git")
        .args(["fetch", "origin", branch])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("failed to execute git fetch");
    if !status.success() {
        return Err(Error::from_str("git fetch failed"));
    }

    // Below is an alternative way using git2 crate, but it requires setting up SSH credentials.
    // let mut cb = RemoteCallbacks::new();
    // cb.credentials(|_url, username_from_url, _allowed_types| {
    //     if let Some(home) = home_dir() {
    //         let private_key = home.join(".ssh").join("id_rsa");
    //         if private_key.exists() {
    //             return Cred::ssh_key(username_from_url.unwrap_or("git"), None, &private_key, None);
    //         }
    //     }
    //     Cred::default()
    // });
    // let mut fo = FetchOptions::new();
    // fo.remote_callbacks(cb);

    // let mut remote = repo.find_remote("origin")?;
    // remote.fetch(&[branch], Some(&mut fo), None)?;

    let remote_ref = repo.find_reference(&format!("refs/remotes/origin/{}", branch))?;
    let remote_commit = remote_ref.peel_to_commit()?;
    let local_commit = head.peel_to_commit()?;
    Ok(remote_commit.id() == local_commit.id())
}

pub fn create_tag(tag: &str) -> Result<(), Error> {
    let repo = Repository::open(".")?;
    if let Ok(existing_tag) = repo.find_reference(&format!("refs/tags/{}", tag)) {
        let target = existing_tag
            .target()
            .map(|t| t.to_string())
            .unwrap_or("unknown".to_string());
        return Err(Error::from_str(&format!(
            "Tag '{}' already exists at {}",
            tag, target
        )));
    }
    let obj = repo.head()?.peel(git2::ObjectType::Commit)?;
    repo.tag_lightweight(tag, &obj, false)?;
    Ok(())
}
