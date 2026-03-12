use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow{
    pub maker:Pubkey,
    pub mint_a:Pubkey,
    pub mint_b:Pubkey,
    pub maker_mint_a_ata:Pubkey,
    pub vault:Pubkey,
    pub bump:u8,
    pub recieve:u64,
    pub seed:u64
}