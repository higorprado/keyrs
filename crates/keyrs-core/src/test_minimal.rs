// Minimal test to isolate the match expression type error
use crate::Key;

#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    Passthrough(Key),
    Remapped(Key),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestEnum {
    Passthrough(Key),
}

impl TestEnum {
    pub fn from_variant(result: TestResult) -> Self {
        match result {
            TestResult::Passthrough(key) => Self::Passthrough(key),
            TestResult::Remapped(key) => Self::Passthrough(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_pattern() {
        let result = TestResult::Passthrough(Key::from(30));
        let output = TestEnum::from_variant(result);

        // This should work - passing value directly
        assert_eq!(output, TestEnum::Passthrough(Key::from(30)));
    }
}
