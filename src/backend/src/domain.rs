use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCategory {
    Normal,
    R18,
    Nsfw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R18Policy {
    Exclude,
    IncludeBlurred,
    IncludeVisible,
    OnlyR18,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSource {
    Single,
    Bookmark,
    Author,
    Top10,
    Random,
    Smart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    Single,
    Bookmark,
    Author,
    Top10,
    Random,
    Smart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    CompletedWithErrors,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskItemStatus {
    Discovered,
    DuplicateSkipped,
    Downloading,
    Saved,
    ItemFailed,
    PolicySkipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadItemStatus {
    Saved,
    SkippedDuplicate,
    SkippedByPolicy,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivPage {
    pub page_index: u32,
    pub original_url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivWork {
    pub pixiv_id: String,
    pub title: Option<String>,
    pub author_uid: Option<String>,
    pub author_name: Option<String>,
    pub tags: Vec<String>,
    pub category: ImageCategory,
    pub pages: Vec<PixivPage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivWorkRef {
    pub pixiv_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRequest {
    pub pixiv_id: String,
    pub page_index: Option<u32>,
    pub source: ImageSource,
    pub r18_policy: R18Policy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadOutcome {
    pub pixiv_id: String,
    pub page_index: u32,
    pub status: DownloadItemStatus,
    pub local_path: Option<PathBuf>,
    pub metadata: Option<PixivWork>,
}

impl R18Policy {
    pub fn from_api(value: &str) -> Option<Self> {
        match value {
            "exclude" => Some(Self::Exclude),
            "include_blurred" => Some(Self::IncludeBlurred),
            "include_visible" => Some(Self::IncludeVisible),
            "only_r18" => Some(Self::OnlyR18),
            _ => None,
        }
    }

    pub fn allows(self, category: ImageCategory) -> bool {
        match self {
            Self::Exclude => category == ImageCategory::Normal,
            Self::IncludeBlurred | Self::IncludeVisible => true,
            Self::OnlyR18 => category == ImageCategory::R18 || category == ImageCategory::Nsfw,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exclude => "exclude",
            Self::IncludeBlurred => "include_blurred",
            Self::IncludeVisible => "include_visible",
            Self::OnlyR18 => "only_r18",
        }
    }
}

impl ImageCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::R18 => "r18",
            Self::Nsfw => "nsfw",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "normal" => Some(Self::Normal),
            "r18" => Some(Self::R18),
            "nsfw" => Some(Self::Nsfw),
            _ => None,
        }
    }
}

impl ImageSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Bookmark => "bookmark",
            Self::Author => "author",
            Self::Top10 => "top10",
            Self::Random => "random",
            Self::Smart => "smart",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "single" => Some(Self::Single),
            "bookmark" => Some(Self::Bookmark),
            "author" => Some(Self::Author),
            "top10" => Some(Self::Top10),
            "random" => Some(Self::Random),
            "smart" => Some(Self::Smart),
            _ => None,
        }
    }
}

impl TaskType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Bookmark => "bookmark",
            Self::Author => "author",
            Self::Top10 => "top10",
            Self::Random => "random",
            Self::Smart => "smart",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "single" => Some(Self::Single),
            "bookmark" => Some(Self::Bookmark),
            "author" => Some(Self::Author),
            "top10" => Some(Self::Top10),
            "random" => Some(Self::Random),
            "smart" => Some(Self::Smart),
            _ => None,
        }
    }
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::CompletedWithErrors => "completed_with_errors",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "completed_with_errors" => Some(Self::CompletedWithErrors),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::CompletedWithErrors | Self::Failed | Self::Cancelled
        )
    }
}

impl TaskLogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

impl TaskItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::DuplicateSkipped => "duplicate_skipped",
            Self::Downloading => "downloading",
            Self::Saved => "saved",
            Self::ItemFailed => "item_failed",
            Self::PolicySkipped => "policy_skipped",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "discovered" => Some(Self::Discovered),
            "duplicate_skipped" => Some(Self::DuplicateSkipped),
            "downloading" => Some(Self::Downloading),
            "saved" => Some(Self::Saved),
            "item_failed" => Some(Self::ItemFailed),
            "policy_skipped" => Some(Self::PolicySkipped),
            _ => None,
        }
    }
}

impl DownloadItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Saved => "saved",
            Self::SkippedDuplicate => "skipped_duplicate",
            Self::SkippedByPolicy => "skipped_by_policy",
            Self::Failed => "failed",
        }
    }
}
