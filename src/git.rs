use git2::{Error, ObjectType, Repository, StatusOptions};
use semver::Version;

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open() -> Result<Self, Error> {
        let repo = Repository::open(".")?;
        if !repo.head()?.is_branch() {
            return Err(Error::from_str("HEAD is not pointing to a branch"));
        }
        Ok(Self { repo })
    }

    pub fn get_current_branch(&self) -> Result<String, Error> {
        let head = self.repo.head()?;
        Ok(head.shorthand().unwrap_or("unknown").to_string())
    }

    pub fn is_clean_working_tree(&self) -> Result<bool, Error> {
        let mut opts = StatusOptions::new();
        opts.include_ignored(false).include_untracked(false);
        let statuses = self.repo.statuses(Some(&mut opts))?;
        Ok(statuses.is_empty())
    }

    pub fn is_fetch_and_check_clean(&self) -> Result<bool, Error> {
        let head = self.repo.head()?;
        let branch = head.shorthand().unwrap();

        // use git command to fetch
        fetch()?;

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

        // let mut remote = self.repo.find_remote("origin")?;
        // remote.fetch(&[branch], Some(&mut fo), None)?;

        let remote_ref = self
            .repo
            .find_reference(&format!("refs/remotes/origin/{}", branch))?;
        let remote_commit = remote_ref.peel_to_commit()?;
        let local_commit = head.peel_to_commit()?;
        Ok(remote_commit.id() == local_commit.id())
    }

    pub fn create_tag(&self, tag: &str) -> Result<(), Error> {
        if let Ok(existing_tag) = self.repo.find_reference(&format!("refs/tags/{}", tag)) {
            let target = existing_tag
                .target()
                .map(|t| t.to_string())
                .unwrap_or("unknown".to_string());
            return Err(Error::from_str(&format!(
                "Tag '{}' already exists at {}",
                tag, target
            )));
        }
        let obj = self.repo.head()?.peel(ObjectType::Commit)?;
        self.repo.tag_lightweight(tag, &obj, false)?;
        Ok(())
    }

    pub fn get_latest_version_tag(&self) -> Result<Version, Error> {
        let tags = self.repo.tag_names(None)?;

        let mut versions: Vec<Version> = vec![];
        for tag_name in tags.iter().flatten() {
            if let Ok(ver) = Version::parse(tag_name.trim_start_matches('v')) {
                versions.push(ver);
            }
        }
        versions.sort();
        versions.reverse();
        if let Some(latest) = versions.first() {
            Ok(latest.clone())
        } else {
            Err(Error::from_str("No valid semver tags found"))
        }
    }
}

fn fetch() -> Result<(), Error> {
    let status = std::process::Command::new("git")
        .args(["fetch", "origin"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("failed to execute git fetch");
    if !status.success() {
        return Err(Error::from_str("git fetch failed"));
    }
    Ok(())
}

pub fn push_tags(tag: &str) -> Result<(), Error> {
    let status = std::process::Command::new("git")
        .args(["push", "origin", tag])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("failed to execute git push tag");
    if !status.success() {
        return Err(Error::from_str("git push tag failed"));
    }
    Ok(())
}
