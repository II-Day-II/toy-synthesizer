pub trait Linsearch<T> {
    fn search(&self, _: &T) -> Option<usize>;
}
impl<T: Eq> Linsearch<T> for Vec<T> {
    fn search(&self, key: &T) -> Option<usize> {
        for m in self.iter().enumerate() {
            if m.1 == key {
                return Some(m.0)
            }
        }
        None
    }
}