#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SudokuNumber {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl SudokuNumber {
    pub fn to_index(&self) -> usize {
        let number: usize = (*self).into();
        number - 1
    }

    pub fn to_u8(&self) -> u8 {
        let number: usize = (*self).into();
        number as u8
    }
}

impl TryFrom<usize> for SudokuNumber {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(SudokuNumber::One),
            2 => Ok(SudokuNumber::Two),
            3 => Ok(SudokuNumber::Three),
            4 => Ok(SudokuNumber::Four),
            5 => Ok(SudokuNumber::Five),
            6 => Ok(SudokuNumber::Six),
            7 => Ok(SudokuNumber::Seven),
            8 => Ok(SudokuNumber::Eight),
            9 => Ok(SudokuNumber::Nine),
            _ => Err(()),
        }
    }
}

impl From<SudokuNumber> for usize {
    fn from(value: SudokuNumber) -> Self {
        match value {
            SudokuNumber::One => 1,
            SudokuNumber::Two => 2,
            SudokuNumber::Three => 3,
            SudokuNumber::Four => 4,
            SudokuNumber::Five => 5,
            SudokuNumber::Six => 6,
            SudokuNumber::Seven => 7,
            SudokuNumber::Eight => 8,
            SudokuNumber::Nine => 9,
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct SudokuNumbers {
    // false means the number is not contained
    numbers: [bool; 9],
}

impl std::fmt::Debug for SudokuNumbers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_numbers().collect::<Vec<_>>().fmt(f)
    }
}

impl SudokuNumbers {
    pub fn new(numbers: impl IntoIterator<Item = SudokuNumber>) -> Self {
        let mut real_numbers: [bool; 9] = Default::default();
        for index in numbers.into_iter().map(|num| num.to_index()) {
            real_numbers[index] = true;
        }
        Self {
            numbers: real_numbers,
        }
    }

    pub fn new_all() -> Self {
        Self { numbers: [true; 9] }
    }

    pub fn get_numbers(&self) -> impl Iterator<Item = SudokuNumber> {
        self.numbers
            .iter()
            .enumerate()
            .filter(|(_, available)| **available)
            .map(|(index, _)| (index + 1).try_into().unwrap())
    }

    pub fn set_number(&mut self, number: SudokuNumber) {
        self.numbers[number.to_index()] = true;
    }

    pub fn del_number(&mut self, number: SudokuNumber) {
        self.numbers[number.to_index()] = false;
    }

    pub fn has_number(&self, number: SudokuNumber) -> bool {
        self.numbers[number.to_index()]
    }

    pub fn count_numbers(&self) -> usize {
        self.numbers.iter().filter(|f| **f).count()
    }

    pub fn get_missing_numbers(&self) -> impl Iterator<Item = SudokuNumber> {
        self.numbers
            .iter()
            .enumerate()
            .filter(|(_, available)| !**available)
            .map(|(index, _)| (index + 1).try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numbers() {
        let mut numbers = SudokuNumbers::new([SudokuNumber::One, SudokuNumber::Seven]);

        assert!(numbers.has_number(SudokuNumber::One));
        assert!(numbers.has_number(SudokuNumber::Seven));

        assert!(!numbers.has_number(SudokuNumber::Two));
        assert!(!numbers.has_number(SudokuNumber::Three));
        assert!(!numbers.has_number(SudokuNumber::Four));
        assert!(!numbers.has_number(SudokuNumber::Five));
        assert!(!numbers.has_number(SudokuNumber::Six));
        assert!(!numbers.has_number(SudokuNumber::Nine));
        assert!(!numbers.has_number(SudokuNumber::Eight));

        assert_eq!(
            numbers.get_numbers().collect::<Vec<SudokuNumber>>(),
            vec![SudokuNumber::One, SudokuNumber::Seven]
        );

        numbers.set_number(SudokuNumber::Eight);
        assert!(numbers.has_number(SudokuNumber::Eight));

        numbers.del_number(SudokuNumber::One);
        assert!(!numbers.has_number(SudokuNumber::One));
    }
}
