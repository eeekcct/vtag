use inquire::{Select, Text};
use semver::Version;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum BumpType {
    Patch,
    Minor,
    Major,
}

impl fmt::Display for BumpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BumpType::Patch => write!(f, "Patch"),
            BumpType::Minor => write!(f, "Minor"),
            BumpType::Major => write!(f, "Major"),
        }
    }
}

pub fn select_bump_type() -> Result<BumpType, inquire::InquireError> {
    let options = vec![BumpType::Patch, BumpType::Minor, BumpType::Major];
    let ans = Select::new("Select version bump:", options).prompt();
    match ans {
        Ok(bump) => Ok(bump),
        Err(e) => Err(e),
    }
}

pub fn bump_version(latest: &Version, bump: BumpType) -> Version {
    let mut new = latest.clone();
    match bump {
        BumpType::Patch => new.patch += 1,
        BumpType::Minor => {
            new.minor += 1;
            new.patch = 0;
        }
        BumpType::Major => {
            new.major += 1;
            new.minor = 0;
            new.patch = 0;
        }
    }
    new
}

pub fn check_create_tag(new: &str, branch: &str) -> bool {
    let msg = format!("Create tag '{}' on branch '{}'? (y/N):", new, branch);
    let ans = Text::new(&msg).prompt();

    match ans {
        Ok(input) => input.to_lowercase() == "y",
        Err(_) => false,
    }
}

pub fn format_vtag(tag: &str) -> String {
    if tag.starts_with('v') {
        tag.to_string()
    } else {
        format!("v{}", tag)
    }
}

pub fn is_valid_tag(tag: &str) -> bool {
    let trimmed = tag.strip_prefix('v');
    match trimmed {
        Some(rest) => Version::parse(rest).is_ok(),
        None => false,
    }
}
