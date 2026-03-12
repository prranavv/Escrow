use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError{
    #[msg("Recieve should be >0")]
    InvalidRecieve,
    #[msg("Amount should be >0")]
    InvalidAmount,
    #[msg("Invalid mint A")]
    InvalidMintA,
    #[msg("Invalid mint B")]
    InvalidMintB,
    #[msg("Invalid maker provided")]
    InvalidMaker
}