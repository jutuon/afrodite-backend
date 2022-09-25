//! Types for files in different repositories



pub struct GitPath<'a>(&'a str);

impl GitPath<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

pub struct LiveVersionPath<'a>(&'a str);

impl LiveVersionPath<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

pub struct TmpPath<'a>(&'a str);

impl TmpPath<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

/// Get file name which is used for committing this file to Git.
pub trait GetGitPath {
    /// Path relative to git repository root.
    fn git_path(&self) -> GitPath<'_>;
}

/// Get file name for file which is ment to be consumed in web requests.
pub trait GetLiveVersionPath {
    /// Path relative to git repository root.
    fn live_path(&self) -> LiveVersionPath<'_>;
}

/// Get file name which is used for creating tmp files which are only used
/// when modifying files with no version history.
pub trait GetTmpPath {
    /// Path relative to git repository root.
    fn tmp_path(&self) -> TmpPath<'_>;
}


/// Files in profile repository
#[derive(Debug, Clone, Copy)]
pub enum CoreFile {
    /// Plain text containing profile ID
    Id,

    /// JSON text file
    ProfileJson,

    /// JSON text file
    PrivateUserInfoJson,
}

impl GetGitPath for CoreFile {
    fn git_path(&self) -> GitPath<'_> {
        GitPath(match self {
            Self::Id => "id.txt.git",
            Self::ProfileJson => "profile.txt.git",
            Self::PrivateUserInfoJson => "user.txt.git",
        })
    }
}

impl GetLiveVersionPath for CoreFile {
    fn live_path(&self) -> LiveVersionPath<'_> {
        LiveVersionPath(match self {
            Self::Id => "id.txt",
            Self::ProfileJson => "profile.txt",
            Self::PrivateUserInfoJson => "user.txt",
        })
    }
}

/// Files not in version history but in profile history.
#[derive(Debug, Clone, Copy)]
pub enum CoreFileNoHistory {
    /// Plain text containing API key
    ApiToken,
}

impl GetTmpPath for CoreFileNoHistory {
    fn tmp_path(&self) -> TmpPath<'_> {
        TmpPath(match self {
            Self::ApiToken => "api_key.txt.tmp",
        })
    }
}

impl GetLiveVersionPath for CoreFileNoHistory {
    fn live_path(&self) -> LiveVersionPath<'_> {
        LiveVersionPath(match self {
            Self::ApiToken => "api_key.txt",
        })
    }
}


// TODO: Append only files (possibly for IP addresses). Set max limit for ip
// address changes or something?
