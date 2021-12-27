use super::Cost;
impl Cost for u8 {
    fn zero() -> Self {
        0
    }
    fn max_val() -> Self {
        Self::MAX
    }
}
impl Cost for u16 {
    fn zero() -> Self {
        0
    }
    fn max_val() -> Self {
        Self::MAX
    }
}
impl Cost for u32 {
    fn zero() -> Self {
        0
    }
    fn max_val() -> Self {
        Self::MAX
    }
}
impl Cost for u64 {
    fn zero() -> Self {
        0
    }
    fn max_val() -> Self {
        Self::MAX
    }
}
