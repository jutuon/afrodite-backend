//! Types related to files





#[derive(Debug)]
pub struct StaticFileName<'a>(&'a str);

impl StaticFileName<'_> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for StaticFileName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub trait GetStaticFileName {
    fn file_name(&self) -> StaticFileName<'static>;
}

#[derive(Debug, Clone, Copy)]
#[repr(i64)]
pub enum ImageSlot {
    Image1 = 0,
    Image2 = 1,
    Image3 = 2,
}

impl GetStaticFileName for ImageSlot {
    fn file_name(&self) -> StaticFileName<'static> {
        StaticFileName(match self {
            Self::Image1 => "slot1.jpg",
            Self::Image2 => "slot2.jpg",
            Self::Image3 => "slot3.jpg",
        })
    }
}

// TODO: Set max limit for IP
// address changes or something (limit IP address history size)?
