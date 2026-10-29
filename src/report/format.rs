//! Version selection for solve reports; independent of CLI and HTTP parsers.

/// Format 1 preserves the original body. Format 2 appends versioned proof metadata.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JsonFormat {
    V1,
    #[default]
    V2,
}

impl std::str::FromStr for JsonFormat {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "1" => Ok(Self::V1),
            "2" => Ok(Self::V2),
            _ => Err("JSON format must be 1 or 2".into()),
        }
    }
}

pub(super) fn git_commit() -> Option<&'static str> {
    option_env!("CUBACLE_GIT_COMMIT").filter(|value| {
        value.len() == 40
            && value
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}
