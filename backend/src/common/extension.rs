pub trait OptionStringExt {
    /// Returns true if the Option is None or if the predicate returns true for the contained String
    fn is_none_or<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(&str) -> bool;

    /// Returns true if the Option is None or if the contained String is empty after trimming
    fn is_none_or_empty(&self) -> bool;
}

impl OptionStringExt for Option<String> {
    fn is_none_or<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(&str) -> bool,
    {
        match self {
            None => true,
            Some(s) => predicate(s),
        }
    }

    fn is_none_or_empty(&self) -> bool {
        self.is_none_or(|s| s.trim().is_empty())
    }
}
