#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub index: usize,
    pub span: (usize, usize),
}

impl Location {
    pub fn new(index: usize, start: usize, end: usize) -> Self {
        Location { index, span: (start, end) }
    }

    pub fn between(start: Location, end: Location) -> Self {
        Location { index: start.index, span: (start.span.0, end.span.1) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Located<T> {
    pub val: T,
    pub loc: Location,
}

pub trait LocatedExt: Sized {
    fn loc_at(self, index: usize, start: usize, end: usize) -> Located<Self> {
        Located { val: self, loc: Location::new(index, start, end.max(start + 1)) }
    }

    fn loc_copy(self, loc: Location) -> Located<Self> {
        Located { val: self, loc }
    }

    fn loc_between(self, start: Location, end: Location) -> Located<Self> {
        Located { val: self, loc: Location::between(start, end) }
    }
}

impl<T> LocatedExt for T {}

pub trait LocatedResultExt<T, E> {
    fn err_loc_at(self, index: usize, start: usize, end: usize) -> Result<T, Located<E>>;
    fn err_loc_copy(self, loc: Location) -> Result<T, Located<E>>;
    fn err_loc_between(self, start: Location, end: Location) -> Result<T, Located<E>>;
}

impl<T, E> LocatedResultExt<T, E> for Result<T, E> {
    fn err_loc_at(self, index: usize, start: usize, end: usize) -> Result<T, Located<E>> {
        self.map_err(|err| err.loc_at(index, start, end))
    }

    fn err_loc_copy(self, loc: Location) -> Result<T, Located<E>> {
        self.map_err(|err| err.loc_copy(loc))
    }

    fn err_loc_between(self, start: Location, end: Location) -> Result<T, Located<E>> {
        self.map_err(|err| err.loc_between(start, end))
    }
}
