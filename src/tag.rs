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
    Select::new("Select version bump:", options).prompt()
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
        Ok(input) => parse_confirm(&input),
        Err(_) => false,
    }
}

pub fn parse_confirm(input: &str) -> bool {
    input.to_lowercase() == "y"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_version_patch() {
        let v = Version::new(1, 2, 3);
        let next = bump_version(&v, BumpType::Patch);
        assert_eq!(next, Version::new(1, 2, 4));
    }

    #[test]
    fn bump_version_minor() {
        let v = Version::new(1, 2, 3);
        let next = bump_version(&v, BumpType::Minor);
        assert_eq!(next, Version::new(1, 3, 0));
    }

    #[test]
    fn bump_version_major() {
        let v = Version::new(1, 2, 3);
        let next = bump_version(&v, BumpType::Major);
        assert_eq!(next, Version::new(2, 0, 0));
    }

    #[test]
    fn format_vtag_prefixed() {
        assert_eq!(format_vtag("v1.0.0"), "v1.0.0");
    }

    #[test]
    fn format_vtag_unprefixed() {
        assert_eq!(format_vtag("1.0.0"), "v1.0.0");
    }

    #[test]
    fn is_valid_tag_good() {
        assert!(is_valid_tag("v1.2.3"));
    }

    #[test]
    fn is_valid_tag_bad() {
        assert!(!is_valid_tag("not-a-tag"));
    }

    #[test]
    fn bumptype_display() {
        assert_eq!(BumpType::Patch.to_string(), "Patch");
        assert_eq!(BumpType::Minor.to_string(), "Minor");
        assert_eq!(BumpType::Major.to_string(), "Major");
    }

    #[test]
    fn parse_confirm_yes() {
        assert!(parse_confirm("y"));
        assert!(parse_confirm("Y"));
    }

    #[test]
    fn parse_confirm_no() {
        assert!(!parse_confirm("n"));
        assert!(!parse_confirm(""));
        assert!(!parse_confirm("yes"));
    }
}
