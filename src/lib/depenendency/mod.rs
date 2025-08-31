#[derive(Debug)]
pub struct Dependency<TFrom: AsRef<str>, TTo: AsRef<str>> {
    from: TFrom,
    to: TTo,
}

impl<TFrom: AsRef<str>, TTo: AsRef<str>> PartialEq for Dependency<TFrom, TTo> {
    fn eq(&self, other: &Self) -> bool {
        self.from.as_ref() == other.from.as_ref() && self.to.as_ref() == other.to.as_ref()
    }
}

impl<TFrom: AsRef<str>, TTo: AsRef<str>> Dependency<TFrom, TTo> {
    pub fn create(from: TFrom, to: TTo) -> Dependency<TFrom, TTo> {
        Dependency { from, to }
    }
}

impl<TType: AsRef<str>> Dependency<TType, TType> {
    pub fn is_internal(&self) -> bool {
       self.from.as_ref() == self.to.as_ref()
    }
}
