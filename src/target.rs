//!
//! The Yul compiler target.
//!

use std::convert::TryFrom;

///
/// The Yul compiler target.
///
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// The x86 target.
    x86,
    /// The zkEVM assembly target.
    zkEVM,
}

impl TryFrom<&str> for Target {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(match input {
            "x86" => Self::x86,
            "zkevm" => Self::zkEVM,

            _ => return Err(input.to_owned()),
        })
    }
}

impl From<Option<&inkwell::targets::TargetMachine>> for Target {
    fn from(machine: Option<&inkwell::targets::TargetMachine>) -> Self {
        match machine {
            Some(machine) => {
                if machine.get_target().get_name().to_string_lossy().as_ref()
                    == compiler_common::vm::TARGET_NAME
                {
                    Self::zkEVM
                } else {
                    Self::x86
                }
            }
            None => Self::x86,
        }
    }
}
