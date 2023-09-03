//! Data Used when estimates were generated
use crate::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataUsedUnitary {
    /// Undifferenced Phases were used
    UndifferencedPhase,
    /// d/dt( UndifferencedPhase) was used
    UndifferencedPhaseDerivative,
    /// 2 Receiver / 1 Sat carrier phase
    DualReceiverPhase,
    /// d/dt(DualReceiverPhase) was used
    DualReceiverPhaseDerivative,
    /// 2 Receiver / 2 Sat carrier phase
    DualReceiverDualPhase,
    /// d/dt(DualReceiverDualPhase) was used
    DualReceiverDualPhaseDerivative,
    /// Undifferenced Code
    UndifferencedCode,
    /// d/dt(UndifferencedCode) was used
    UndifferencedCodeDerivative,
    /// 2 Receiver / 1 Sat code phase
    DualReceiverCode,
    /// d/dt(DualReceiverCode) was used
    DualReceiverCodeDerivative,
    /// 2 Receiver / 2 Sat code phase
    DualReceiverDualCode,
    /// d/dt(DualReceiverDualCode) was used
    DualReceiverDualCodeDerivative,
    /// Complex combination was used
    ComplexMix,
    #[default]
    /// A combination of Orbits from several agencies was used
    Orbit,
}

impl std::str::FromStr for DataUsedUnitary {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("u") {
            Ok(Self::UndifferencedPhase)
        } else if s.eq("du") {
            Ok(Self::UndifferencedPhaseDerivative)
        } else if s.eq("s") {
            Ok(Self::DualReceiverPhase)
        } else if s.eq("ds") {
            Ok(Self::DualReceiverPhaseDerivative)
        } else if s.eq("d") {
            Ok(Self::DualReceiverDualPhase)
        } else if s.eq("dd") {
            Ok(Self::DualReceiverDualPhaseDerivative)
        } else if s.eq("U") {
            Ok(Self::UndifferencedCode)
        } else if s.eq("dU") {
            Ok(Self::UndifferencedCodeDerivative)
        } else if s.eq("S") {
            Ok(Self::DualReceiverCode)
        } else if s.eq("dS") {
            Ok(Self::DualReceiverCodeDerivative)
        } else if s.eq("D") {
            Ok(Self::DualReceiverDualCode)
        } else if s.eq("dD") {
            Ok(Self::DualReceiverDualCodeDerivative)
        } else if s.to_ascii_lowercase().eq("mixed") {
            Ok(Self::ComplexMix)
        } else if s.to_ascii_lowercase().eq("orbit") {
            Ok(Self::Orbit)
        } else {
            Err(ParsingError::DataUsedUnitary(s.to_string()))
        }
    }
}

impl std::fmt::Display for DataUsedUnitary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UndifferencedPhase => f.write_str("u"),
            Self::UndifferencedPhaseDerivative => f.write_str("du"),
            Self::DualReceiverPhase => f.write_str("s"),
            Self::DualReceiverPhaseDerivative => f.write_str("ds"),
            Self::DualReceiverDualPhase => f.write_str("d"),
            Self::DualReceiverDualPhaseDerivative => f.write_str("dd"),
            Self::UndifferencedCode => f.write_str("U"),
            Self::UndifferencedCodeDerivative => f.write_str("dU"),
            Self::DualReceiverCode => f.write_str("S"),
            Self::DualReceiverCodeDerivative => f.write_str("dS"),
            Self::DualReceiverDualCode => f.write_str("D"),
            Self::DualReceiverDualCodeDerivative => f.write_str("dD"),
            Self::ComplexMix => f.write_str("MIXED"),
            Self::Orbit => f.write_str("ORBIT"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataUsed {
    inner: Vec<DataUsedUnitary>,
}

impl std::str::FromStr for DataUsed {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let content = s.trim();
        if content.eq("MIXED") {
            Ok(Self {
                inner: vec![DataUsedUnitary::ComplexMix],
            })
        } else if content.contains('+') {
            let offset = content.find('+').unwrap();
            Ok(Self {
                inner: vec![
                    DataUsedUnitary::from_str(&content[..offset])?,
                    DataUsedUnitary::from_str(&content[offset + 1..])?,
                ],
            })
        } else {
            Ok(Self {
                inner: vec![DataUsedUnitary::from_str(content)?],
            })
        }
    }
}

impl std::fmt::Display for DataUsed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let len = self.inner.len();
        if len == 1 {
            f.write_str(&format!("{}", self.inner[0]))
        } else if len > 1 {
            f.write_str(&format!("{}+{}", self.inner[0], self.inner[1]))
        } else {
            Ok(())
        }
    }
}

impl DataUsed {
    pub fn complex_combination(&self) -> bool {
        if self.inner.is_empty() {
            self.inner[0] == DataUsedUnitary::ComplexMix
        } else {
            false
        }
    }
    pub fn combination(&self) -> Option<(DataUsedUnitary, DataUsedUnitary)> {
        if self.inner.len() == 2 {
            Some((self.inner[0], self.inner[1]))
        } else {
            None
        }
    }
    pub fn single(&self) -> Option<DataUsedUnitary> {
        if self.inner.len() == 1 {
            Some(self.inner[0])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::DataUsed;
    use super::DataUsedUnitary;
    use std::str::FromStr;
    #[test]
    fn unitary_from_str() {
        for (code, expected) in vec![
            ("u", DataUsedUnitary::UndifferencedPhase),
            ("du", DataUsedUnitary::UndifferencedPhaseDerivative),
            ("s", DataUsedUnitary::DualReceiverPhase),
            ("ds", DataUsedUnitary::DualReceiverPhaseDerivative),
            ("d", DataUsedUnitary::DualReceiverDualPhase),
            ("dd", DataUsedUnitary::DualReceiverDualPhaseDerivative),
            ("U", DataUsedUnitary::UndifferencedCode),
            ("dU", DataUsedUnitary::UndifferencedCodeDerivative),
            ("S", DataUsedUnitary::DualReceiverCode),
            ("dS", DataUsedUnitary::DualReceiverCodeDerivative),
            ("D", DataUsedUnitary::DualReceiverDualCode),
            ("dD", DataUsedUnitary::DualReceiverDualCodeDerivative),
            ("mixed", DataUsedUnitary::ComplexMix),
            ("orbit", DataUsedUnitary::Orbit),
        ] {
            assert_eq!(
                DataUsedUnitary::from_str(code),
                Ok(expected),
                "failed to parse {} from \"{}\"",
                expected,
                code
            );
        }
    }
    #[test]
    fn combination_from_str() {
        for (code, expected) in vec![
            (
                "u",
                DataUsed {
                    inner: vec![DataUsedUnitary::UndifferencedPhase],
                },
            ),
            (
                "U",
                DataUsed {
                    inner: vec![DataUsedUnitary::UndifferencedCode],
                },
            ),
            (
                "du",
                DataUsed {
                    inner: vec![DataUsedUnitary::UndifferencedPhaseDerivative],
                },
            ),
            (
                "dU",
                DataUsed {
                    inner: vec![DataUsedUnitary::UndifferencedCodeDerivative],
                },
            ),
            (
                "u+du",
                DataUsed {
                    inner: vec![
                        DataUsedUnitary::UndifferencedPhase,
                        DataUsedUnitary::UndifferencedPhaseDerivative,
                    ],
                },
            ),
            (
                "U+dU",
                DataUsed {
                    inner: vec![
                        DataUsedUnitary::UndifferencedCode,
                        DataUsedUnitary::UndifferencedCodeDerivative,
                    ],
                },
            ),
            (
                "u+U",
                DataUsed {
                    inner: vec![
                        DataUsedUnitary::UndifferencedPhase,
                        DataUsedUnitary::UndifferencedCode,
                    ],
                },
            ),
            (
                "orbit",
                DataUsed {
                    inner: vec![DataUsedUnitary::Orbit],
                },
            ),
        ] {
            assert_eq!(
                DataUsed::from_str(code),
                Ok(expected.clone()),
                "failed to parse {} from \"{}\"",
                expected,
                code
            );
        }
    }
}
