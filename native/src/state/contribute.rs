
use super::utils::{DataLen, Initialized};

#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Contribute {
    pub is_initialized: bool,
    pub amount: u64,
}

impl DataLen for Contribute {
    const LEN: usize = core::mem::size_of::<Contribute>();
}

impl Initialized for Contribute {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Contribute {
    pub const SEED: &'static str = "contribute";

    pub fn initialize (
        &mut self,
        amount: u64,
    ) {
        self.is_initialized = true;
        self.amount =  amount;
    }
}