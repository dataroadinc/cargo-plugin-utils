//! Common helper functions shared across cargo plugins.

use std::env;

use anyhow::{
    Context,
    Result,
};
use cargo_metadata::MetadataCommand;

/// Detect GitHub repository from environment or git remote.
#[allow(clippy::disallowed_methods)] // CLI tool needs direct env access
pub fn detect_repo() -> Result<(String, String)> {
    // Try GITHUB_REPOSITORY env var first (set by GitHub Actions)
    if let Ok(repo) = env::var("GITHUB_REPOSITORY") {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Try to detect from git remote
    let repo = gix::discover(".").context("Failed to discover git repository")?;
    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .context("Failed to find default remote")?
        .context("No default remote found")?;

    let remote_url = remote
        .url(gix::remote::Direction::Fetch)
        .context("Failed to get remote URL")?;

    // Parse git@github.com:owner/repo.git or https://github.com/owner/repo.git
    let url_str = remote_url.to_string();
    if let Some(rest) = url_str.strip_prefix("git@github.com:") {
        let rest_trimmed: &str = rest.strip_suffix(".git").unwrap_or(rest);
        let parts: Vec<&str> = rest_trimmed.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    } else if let Some(rest) = url_str.strip_prefix("https://github.com/") {
        let rest_trimmed: &str = rest.strip_suffix(".git").unwrap_or(rest);
        let parts: Vec<&str> = rest_trimmed.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    anyhow::bail!(
        "Could not detect GitHub repository. Set GITHUB_REPOSITORY or use --owner/--repo flags"
    );
}

/// Get owner and repo from args or environment.
pub fn get_owner_repo(owner: Option<String>, repo: Option<String>) -> Result<(String, String)> {
    match (owner, repo) {
        (Some(o), Some(r)) => Ok((o, r)),
        (Some(_), None) | (None, Some(_)) => {
            anyhow::bail!("Both --owner and --repo must be provided together");
        }
        (None, None) => detect_repo(),
    }
}

/// Find the Cargo package using cargo_metadata.
///
/// This automatically respects Cargo's `--manifest-path` option when running
/// as a cargo subcommand.
///
/// Returns the package that corresponds to the current context, in order:
/// 1. Package whose directory matches the current working directory
/// 2. Package whose manifest path matches `current_dir/Cargo.toml`
/// 3. Root package (if workspace has a root package)
/// 4. First default-member (if workspace has default-members configured)
/// 5. Error if no package can be determined
pub fn find_package(manifest_path: Option<&std::path::Path>) -> Result<cargo_metadata::Package> {
    let mut cmd = MetadataCommand::new();
    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd.exec().context("Failed to get cargo metadata")?;

    // Try to find the package in the current working directory
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Canonicalize current directory and all package directories, then find match
    let canonical_current_dir = current_dir.canonicalize().ok();
    let packages_with_dirs: Vec<_> = metadata
        .packages
        .iter()
        .filter_map(|pkg| {
            // Get the directory containing the manifest (package directory)
            pkg.manifest_path
                .as_std_path()
                .parent()
                .and_then(|p| p.canonicalize().ok())
                .map(|p| (pkg.clone(), p))
        })
        .collect();

    // Try to match current directory with a package directory
    if let Some(ref canonical_current) = canonical_current_dir
        && let Some((pkg, _)) = packages_with_dirs
            .iter()
            .find(|(_, pkg_dir)| pkg_dir == canonical_current)
    {
        return Ok(pkg.clone());
    }

    // Also try matching the manifest path directly (for cases where Cargo.toml is
    // in current dir)
    let current_manifest = current_dir.join("Cargo.toml");
    let canonical_current_manifest = current_manifest.canonicalize().ok();
    let packages_with_manifests: Vec<_> = metadata
        .packages
        .iter()
        .filter_map(|pkg| {
            pkg.manifest_path
                .as_std_path()
                .canonicalize()
                .ok()
                .map(|p| (pkg.clone(), p))
        })
        .collect();

    if let Some(ref canonical) = canonical_current_manifest
        && let Some((pkg, _)) = packages_with_manifests
            .iter()
            .find(|(_, pkg_path)| pkg_path == canonical)
    {
        return Ok(pkg.clone());
    }

    // Fallback to root package (workspace root or single package)
    if let Some(root_package) = metadata.root_package() {
        return Ok(root_package.clone());
    }

    // If we're in a workspace without a root package, check for default-members
    // This follows cargo's behavior: use default-members if available
    // workspace_default_members implements Deref<Target = [PackageId]>, so we can
    // use it as a slice It may not be available in older Cargo versions, so we
    // check if it's available first
    if metadata.workspace_default_members.is_available()
        && !metadata.workspace_default_members.is_empty()
        && let Some(first_default_id) = metadata.workspace_default_members.first()
        && let Some(default_package) = metadata
            .packages
            .iter()
            .find(|pkg| &pkg.id == first_default_id)
    {
        return Ok(default_package.clone());
    }

    // If no default-members, we need to be in a package directory
    anyhow::bail!(
        "No package found in current directory. Run this command from a package directory, \
         or use --manifest-path to specify a package."
    )
}

/// Get package version from a specific manifest path using cargo_metadata.
pub fn get_package_version_from_manifest(manifest_path: &std::path::Path) -> Result<String> {
    let package = find_package(Some(manifest_path))?;
    Ok(package.version.to_string())
}

/// Get cargo metadata for a workspace or package.
///
/// This is a convenience function that handles `--manifest-path` idiomatically.
/// When running as a cargo subcommand, cargo passes `--manifest-path` to the
/// subcommand, so this function handles it explicitly.
pub fn get_metadata(manifest_path: Option<&std::path::Path>) -> Result<cargo_metadata::Metadata> {
    let mut cmd = MetadataCommand::new();
    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }
    cmd.exec().context("Failed to get cargo metadata")
}

/// Get all workspace packages.
///
/// Returns all packages in the workspace (supports both single-package projects
/// and workspace projects with packages in crates/ or elsewhere).
pub fn get_workspace_packages(
    manifest_path: Option<&std::path::Path>,
) -> Result<Vec<cargo_metadata::Package>> {
    let metadata = get_metadata(manifest_path)?;
    Ok(metadata.packages)
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_get_owner_repo_both_provided() {
        let result = get_owner_repo(Some("owner".to_string()), Some("repo".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ("owner".to_string(), "repo".to_string()));
    }

    #[test]
    fn test_get_owner_repo_only_owner() {
        let result = get_owner_repo(Some("owner".to_string()), None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Both --owner and --repo must be provided")
        );
    }

    #[test]
    fn test_get_owner_repo_only_repo() {
        let result = get_owner_repo(None, Some("repo".to_string()));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Both --owner and --repo must be provided")
        );
    }

    #[test]
    fn test_get_owner_repo_from_env() {
        // Save original value if it exists
        let original = env::var("GITHUB_REPOSITORY").ok();

        // Test GITHUB_REPOSITORY env var
        unsafe {
            env::set_var("GITHUB_REPOSITORY", "test-owner/test-repo");
        }
        let result = get_owner_repo(None, None);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ("test-owner".to_string(), "test-repo".to_string())
        );

        // Restore original value
        unsafe {
            if let Some(val) = original {
                env::set_var("GITHUB_REPOSITORY", &val);
            } else {
                env::remove_var("GITHUB_REPOSITORY");
            }
        }
    }

    #[test]
    fn test_get_owner_repo_invalid_env() {
        // Test invalid GITHUB_REPOSITORY format
        unsafe {
            env::set_var("GITHUB_REPOSITORY", "invalid");
        }
        let _result = get_owner_repo(None, None);
        // Should fail if not in a git repo or invalid format
        unsafe {
            env::remove_var("GITHUB_REPOSITORY");
        }
    }

    #[test]
    fn test_find_package_in_current_dir() {
        // This test requires being in a directory with a Cargo.toml
        // We'll test that it doesn't panic, but actual success depends on environment
        let result = find_package(None);
        // Either succeeds (if in a cargo project) or fails with a descriptive error
        if let Err(e) = result {
            assert!(e.to_string().contains("package") || e.to_string().contains("manifest"));
        }
    }

    #[test]
    fn test_find_package_with_manifest_path() {
        // Test with a non-existent manifest path
        let result = find_package(Some(std::path::Path::new("/nonexistent/path/Cargo.toml")));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_package_version_from_manifest() {
        // Test with a non-existent manifest path
        let result =
            get_package_version_from_manifest(std::path::Path::new("/nonexistent/path/Cargo.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_repo_from_env() {
        // Save original value if it exists
        let original = env::var("GITHUB_REPOSITORY").ok();

        unsafe {
            env::set_var("GITHUB_REPOSITORY", "env-owner/env-repo");
        }
        let result = detect_repo();
        // Should succeed because GITHUB_REPOSITORY is set and takes precedence
        assert!(result.is_ok());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "env-owner");
        assert_eq!(repo, "env-repo");

        // Restore original value
        unsafe {
            if let Some(val) = original {
                env::set_var("GITHUB_REPOSITORY", &val);
            } else {
                env::remove_var("GITHUB_REPOSITORY");
            }
        }
    }

    #[test]
    fn test_detect_repo_invalid_env_format() {
        unsafe {
            env::set_var("GITHUB_REPOSITORY", "invalid-format");
        }
        let _result = detect_repo();
        // Should fail if not in a git repo
        unsafe {
            env::remove_var("GITHUB_REPOSITORY");
        }
    }
}
