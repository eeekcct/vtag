use git2::{Error, Repository, StatusOptions};
use octocrab::Octocrab;
use once_cell::sync::Lazy;
use regex::Regex;
use semver::Version;

static GITHUB_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        (?:https://github\.com/|git@github\.com:)
        (?P<owner>[^/]+)/
        (?P<repo>[^/.]+)(?:\.git)?$",
    )
    .expect("Failed to compile GitHub URL regex")
});

pub struct GitRepo {
    repo: Repository,
}

pub struct GitApi {
    owner: String,
    repo: String,
    octocrab: Octocrab,
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
        Ok(head.shorthand()?.to_string())
    }

    pub fn is_clean_working_tree(&self) -> Result<bool, Error> {
        let mut opts = StatusOptions::new();
        opts.include_ignored(false).include_untracked(false);
        let statuses = self.repo.statuses(Some(&mut opts))?;
        Ok(statuses.is_empty())
    }

    pub fn is_fetch_and_check_clean(&self) -> Result<bool, Error> {
        let head = self.repo.head()?;
        let branch = head.shorthand()?;

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

        // Create a signed tag using git command
        // git2 Repository does not directly support creating signed tags
        // let obj = self.repo.head()?.peel(ObjectType::Commit)?;
        // self.repo.tag_lightweight(tag, &obj, false)?;
        let status = std::process::Command::new("git")
            .args(["tag", "-s", tag, "-m", tag])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("failed to execute git tag");
        if !status.success() {
            return Err(Error::from_str("git tag command failed"));
        }
        Ok(())
    }

    pub fn get_latest_version_tag(&self) -> Result<Version, Error> {
        let tags = self.repo.tag_names(None)?;

        let mut versions: Vec<Version> = vec![];
        for tag_name in tags.iter().filter_map(Result::ok).flatten() {
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

    pub fn get_repo_owner_name(&self) -> Result<(String, String), Error> {
        let remote = self.repo.find_remote("origin")?;
        let url = remote.url()?;
        let (owner, repo) = parse_github_url(url)?;
        Ok((owner, repo))
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

fn parse_github_url(url: &str) -> Result<(String, String), Error> {
    // https://github.com/owner/repo.git
    // git@github.com:owner/repo.git
    let caps = GITHUB_URL_REGEX
        .captures(url)
        .ok_or_else(|| Error::from_str("Invalid GitHub URL"))?;
    let owner = caps
        .name("owner")
        .ok_or_else(|| Error::from_str("Owner not found"))?
        .as_str()
        .to_string();
    let repo = caps
        .name("repo")
        .ok_or_else(|| Error::from_str("Repo not found"))?
        .as_str()
        .to_string();
    Ok((owner, repo))
}

fn get_token() -> String {
    // Try to get token from environment variables
    if let Ok(env_token) = std::env::var("GITHUB_TOKEN") {
        return env_token;
    } else if let Ok(env_token) = std::env::var("GH_TOKEN") {
        return env_token;
    }

    // Try to get token from gh cli
    let output = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok();
    if let Some(output) = output {
        if output.status.success() {
            if let Ok(gh_token) = String::from_utf8(output.stdout) {
                return gh_token.trim().to_string();
            }
        }
    }
    String::new() // Return empty string if no token found
}

impl GitApi {
    pub fn new(owner: String, repo: String) -> Result<Self, octocrab::Error> {
        let token = get_token();

        let octocrab = Octocrab::builder().personal_token(token).build()?;

        Ok(Self {
            owner,
            repo,
            octocrab,
        })
    }

    pub async fn publish_release(&self, tag: &str) -> Result<(), octocrab::Error> {
        let repo = self.octocrab.repos(self.owner.clone(), self.repo.clone());

        let notes = repo.releases().generate_release_notes(tag).send().await?;

        // Create release with auto-generated release notes
        repo.releases()
            .create(tag)
            .name(tag)
            .body(&notes.body)
            .draft(false)
            .prerelease(false)
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_metadata_uses_git2_result_accessors() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("vtag-test-{}-{unique}", std::process::id()));
        {
            let mut options = git2::RepositoryInitOptions::new();
            options.initial_head("main");
            let repo = Repository::init_opts(&path, &options).unwrap();
            let signature = git2::Signature::now("Test", "test@example.invalid").unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &signature, &signature, "Test", &tree, &[])
                .unwrap();
            drop(tree);
            let commit = repo.head().unwrap().peel(git2::ObjectType::Commit).unwrap();
            for tag in ["v1.2.3", "not-semver", "v2.0.0"] {
                repo.tag_lightweight(tag, &commit, false).unwrap();
            }
            drop(commit);
            repo.remote("origin", "https://github.com/example/example.git")
                .unwrap();
            let git = GitRepo { repo };
            assert_eq!(git.get_current_branch().unwrap(), "main");
            assert_eq!(git.get_latest_version_tag().unwrap(), Version::new(2, 0, 0));
            assert_eq!(
                git.get_repo_owner_name().unwrap(),
                ("example".to_string(), "example".to_string())
            );
        }
        std::fs::remove_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn personal_token_requests_work_without_rustcrypto_rsa() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::time::Duration;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                let count = stream.read(&mut buffer).unwrap();
                assert!(count > 0, "request ended before its headers");
                request.extend_from_slice(&buffer[..count]);
            }
            let body = r#"{"login":"test-user"}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
            String::from_utf8(request).unwrap()
        });

        let client = Octocrab::builder()
            .base_uri(format!("http://{address}"))
            .unwrap()
            .personal_token("test-token".to_string())
            .build()
            .unwrap();
        let response: std::collections::BTreeMap<String, String> =
            client.get("/user", None::<&()>).await.unwrap();
        assert_eq!(response.get("login").unwrap(), "test-user");
        let request = server.join().unwrap().to_ascii_lowercase();
        assert!(request.starts_with("get /user "));
        assert!(request.contains("authorization: bearer test-token\r\n"));
    }

    #[test]
    fn test_parse_github_url() {
        let url = vec![
            "https://github.com/owner/repo.git",
            "git@github.com:owner/repo.git",
        ];
        for u in url {
            let (owner, repo) = parse_github_url(u).unwrap();
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
        }
    }

    #[test]
    fn test_parse_github_url_invalid() {
        let url = vec![
            "https://gitlab.com/owner/repo.git",
            "git@gitlab.com:owner/repo.git",
        ];
        for u in url {
            let result = parse_github_url(u);
            assert!(result.is_err());
        }
    }
}
