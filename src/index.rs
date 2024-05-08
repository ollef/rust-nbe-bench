#[derive(Copy, Clone)]
pub struct Index(pub usize);

#[derive(Clone, Copy)]
pub struct Level(pub usize);

impl Index {
    pub fn to_int(self) -> usize {
        let Index(result) = self;
        result
    }
}

impl std::ops::Add<usize> for Level {
    type Output = Level;

    fn add(self, rhs: usize) -> Self::Output {
        Level(self.to_int() + rhs)
    }
}

impl Level {
    pub fn to_int(self) -> usize {
        let Level(result) = self;
        result
    }

    pub fn to_index(self, base_level: Level) -> Index {
        Index(base_level.to_int() - self.to_int() - 1)
    }
}
