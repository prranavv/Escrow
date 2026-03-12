use anchor_lang::prelude::*;
use anchor_spl::token_2022::{CloseAccount, TransferChecked};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use crate::errors::EscrowError;
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Refund<'info>{
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a:InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint =mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_mint_a_ata:InterfaceAccount<'info,TokenAccount>,
    #[account(
        mut,
        close=maker,
        seeds=[b"escrow",maker.key().as_ref(),escrow.seed.to_le_bytes().as_ref()],
        bump=escrow.bump,
        has_one = maker @ EscrowError::InvalidMaker,
        has_one = mint_a @ EscrowError::InvalidMintA,
    )]
    pub escrow:Account<'info,Escrow>,
    #[account(mut)]
    pub vault:InterfaceAccount<'info,TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl <'info> Refund<'info>{
    pub fn refund(&mut self)->Result<()>{
        let decimals= self.mint_a.decimals;
        let signer_seeds: &[&[&[u8]]] = &[&[b"escrow",self.maker.key.as_ref(),&self.escrow.seed.to_le_bytes(),&[self.escrow.bump]]];
        let cpi_accounts = TransferChecked{
            mint:self.mint_a.to_account_info(),
            from:self.vault.to_account_info(),
            to:self.maker_mint_a_ata.to_account_info(),
            authority:self.escrow.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token_interface::transfer_checked(cpi_context, self.vault.amount, decimals)?;
        Ok(())
    }

    pub fn close_vault(&mut self)->Result<()>{
        let signer_seeds: &[&[&[u8]]] = &[&[b"escrow",self.maker.key.as_ref(),&self.escrow.seed.to_le_bytes(),&[self.escrow.bump]]];
        let close_accounts = CloseAccount{
            account:self.vault.to_account_info(),
            destination:self.maker.to_account_info(),
            authority:self.escrow.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();

        let cpi_context = CpiContext::new_with_signer(cpi_program, close_accounts, signer_seeds);
        token_interface::close_account(cpi_context)?;
        Ok(())
    }
}

pub fn handler(ctx:Context<Refund>)->Result<()>{
    ctx.accounts.refund()?;
    ctx.accounts.close_vault()
}