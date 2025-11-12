#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaybeUsize(usize);

impl MaybeUsize {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn unwrap(&self) -> usize {
        match self.0 {
            usize::MAX => panic!("Called unwrap() on MaybeUsize::None"),
            n => n,
        }
    }

    pub fn inner(&self) -> Option<usize> {
        match self.0 {
            usize::MAX => None,
            n => Some(n),
        }
    }

    pub fn is_some(&self) -> bool {
        self.0 != usize::MAX
    }

    pub fn is_none(&self) -> bool {
        self.0 == usize::MAX
    }
}

impl Default for MaybeUsize {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

impl From<usize> for MaybeUsize {
    fn from(value: usize) -> Self {
        Self(value)
    }
}