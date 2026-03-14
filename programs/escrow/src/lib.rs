use anchor_lang::prelude::*;
pub mod instructions;
pub mod state;
pub mod errors;

pub use instructions::*;

declare_id!("F787RkPTqsZe4ZJUijreGS42hKrznXrvkNJ3P7d97j7s");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,seed:u64,recieve:u64,amount:u64) -> Result<()> {
        instructions::initialize::handler(ctx, recieve, seed, amount)
    }

    pub fn take(ctx:Context<Take>)->Result<()>{
        instructions::take::handler(ctx)
    }

    pub fn refund(ctx:Context<Refund>)->Result<()>{
        instructions::refund::handler(ctx)
    }
}
