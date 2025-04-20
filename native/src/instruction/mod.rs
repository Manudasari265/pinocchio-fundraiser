use pinocchio::program_error::ProgramError;

pub mod initialize;
pub mod contribute;
pub mod check;
pub mod refund;

pub use initialize::*;
pub use contribute::*;
pub use check::*;
pub use refund::*;


#[repr(u8)]
pub enum FundraiserInsruction {
    Initialize,
    Contribute,
    CheckContribution,
    RefundAndClose,
}

impl TryFrom<&u8> for FundraiserInsruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(FundraiserInsruction::Initialize),
            1 => Ok(FundraiserInsruction::Contribute),
            2 => Ok(FundraiserInsruction::CheckContribution),
            3 => Ok(FundraiserInsruction::RefundAndClose),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}