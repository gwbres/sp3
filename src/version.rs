//! sp3 version

use crate::Errors;

#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Version {
    C,
    #[default]
    D,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::C => f.write_str("c"),
            Self::D => f.write_str("d"),
        }
    }
}

impl std::str::FromStr for Version {
    type Err = Errors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("d") {
            Ok(Self::D)
        } else if s.eq("c") {
            Ok(Self::C)
        } else {
            Err(Errors::UnknownVersion(s.to_string()))
        }
    }
}

impl From<Version> for u8 {
    fn from(val: Version) -> Self {
        match val {
            Version::D => 4,
            Version::C => 3,
        }
    }
}

impl From<u8> for Version {
    fn from(lhs: u8) -> Version {
        match lhs {
            4..=u8::MAX => Version::D,
            0..=3 => Version::C,
        }
    }
}

impl std::ops::Add<u8> for Version {
    type Output = Self;
    fn add(self, rhs: u8) -> Self {
        let s: u8 = self.into();
        (s + rhs).into()
    }
}

impl std::ops::Sub<u8> for Version {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self {
        let s: u8 = self.into();
        (s - rhs).into()
    }
}

#[cfg(test)]
mod test {
    use super::Version;
    use std::str::FromStr;
    #[test]
    fn version() {
        for (desc, expected) in vec![("c", Version::C), ("d", Version::D)] {
            assert!(
                Version::from_str(desc).is_ok(),
                "failed to parse Version from \"{}\"",
                desc
            );
        }

        for (vers, expected) in vec![(Version::C, 3), (Version::D, 4)] {
            let version: u8 = vers.into();
            assert_eq!(version, expected, "convertion to integer failed");
        }

        assert!(Version::C < Version::D);
        assert!(Version::D >= Version::C);

        let version: Version = 4_u8.into();
        assert_eq!(version, Version::D);
        assert_eq!(version + 1, Version::D);
        assert_eq!(version - 1, Version::C);

        let version: Version = 3_u8.into();
        assert_eq!(version, Version::C);
        assert_eq!(version + 1, Version::D);
        assert_eq!(version - 1, Version::C);
    }
}
