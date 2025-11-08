use crate::git;
use crate::tag;
use clap::Parser;

#[derive(Parser)]
#[command(
  version = env!("VERSION"),
  about = "Create and push git tags",
  long_about = None,
  long_version = concat!(
    "version ", env!("VERSION"), "\n",
    "  commit: ", env!("COMMIT"), "\n",
    "  built at: ", env!("DATE"), "\n",
    "  rust version: ", env!("RUSTC_VERSION"), "\n",
    "  platform: ", env!("OS"), "/", env!("ARCH")
  ),
)]
struct Args {
    /// Tag name
    tag: Option<String>,

    /// Publish release (default: false)
    #[arg(short, long, default_value_t = false)]
    release: bool,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    cmd(&args.tag, args.release)
}

pub fn cmd(tag: &Option<String>, release: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Open the git repository
    let repo = git::GitRepo::open()?;

    let branch = repo.get_current_branch()?;
    if branch != "main" {
        return Err(Box::from("Not on 'main' branch"));
    }

    if !repo.is_clean_working_tree()? {
        return Err(Box::from("Working tree is not clean"));
    }

    if !repo.is_fetch_and_check_clean()? {
        return Err(Box::from("Local branch is behind remote"));
    }

    let new_tag = if let Some(tag) = tag {
        tag.to_string()
    } else {
        let bump = tag::select_bump_type()?;
        let latest = repo
            .get_latest_version_tag()
            .unwrap_or_else(|_| semver::Version::new(0, 0, 0));
        let new_version = tag::bump_version(&latest, bump);
        tag::format_vtag(&new_version.to_string())
    };

    if !tag::is_valid_tag(&new_tag) {
        return Err(Box::from(format!("Invalid tag name '{}'", new_tag)));
    }

    if !tag::check_create_tag(&new_tag, &branch) {
        println!("🚫 Tag creation cancelled");
        return Ok(());
    }

    repo.create_tag(&new_tag)?;
    git::push_tags(&new_tag)?;
    println!("🚀 Creating and pushing tag '{}'", new_tag);

    if !release {
        return Ok(());
    }

    let (owner, repo) = repo.get_repo_owner_name()?;
    let api = git::GitApi::new(owner, repo)?;
    api.publish_release(&new_tag)?;
    println!("🏷️ Creating release for tag '{}'", new_tag);

    Ok(())
}
