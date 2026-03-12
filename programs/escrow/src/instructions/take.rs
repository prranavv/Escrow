use anchor_lang::prelude::*;
use anchor_spl::token_2022::{CloseAccount, TransferChecked};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use anchor_spl::associated_token::AssociatedToken;
use crate::errors::EscrowError;
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Take<'info>{
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    pub mint_a:InterfaceAccount<'info, Mint>,
    pub mint_b:InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer=taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_mint_a_ata:InterfaceAccount<'info,TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_mint_b_ata:InterfaceAccount<'info,TokenAccount>,
    #[account(
        init_if_needed,
        payer=maker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_mint_b_ata:InterfaceAccount<'info,TokenAccount>,
    #[account(
        mut,
        close=maker,
        seeds=[b"escrow",maker.key().as_ref(),escrow.seed.to_le_bytes().as_ref()],
        bump=escrow.bump,
        has_one = maker @ EscrowError::InvalidMaker,
        has_one = mint_a @ EscrowError::InvalidMintA,
        has_one = mint_b @ EscrowError::InvalidMintB
    )]
    pub escrow:Account<'info,Escrow>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault:InterfaceAccount<'info,TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info,System>
}

impl <'info> Take<'info>{
    pub fn deposit(&mut self)->Result<()>{
        let decimals = self.mint_b.decimals;
        let cpi_accounts = TransferChecked{
            mint:self.mint_b.to_account_info(),
            from:self.taker_mint_b_ata.to_account_info(),
            to:self.maker_mint_b_ata.to_account_info(),
            authority:self.taker.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token_interface::transfer_checked(cpi_context, self.escrow.recieve, decimals)?;
        Ok(())
    }

    pub fn withdraw_and_close_vault(&mut self)->Result<()>{
        require!(self.vault.amount>0,EscrowError::InvalidAmount);
        
        let decimals = self.mint_a.decimals;
        let signer_seeds: &[&[&[u8]]] = &[&[b"escrow",self.maker.key.as_ref(),&self.escrow.seed.to_le_bytes(),&[self.escrow.bump]]];
        let cpi_accounts = TransferChecked{
            mint:self.mint_a.to_account_info(),
            from:self.vault.to_account_info(),
            to:self.taker_mint_a_ata.to_account_info(),
            authority:self.escrow.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);
        token_interface::transfer_checked(cpi_context, self.vault.amount, decimals)?;
        let close_accounts = CloseAccount{
            account:self.vault.to_account_info(),
            destination:self.maker.to_account_info(),
            authority:self.escrow.to_account_info()
        };

        let cpi_context = CpiContext::new_with_signer(self.token_program.to_account_info(), close_accounts, signer_seeds);
        token_interface::close_account(cpi_context)?;
        Ok(())
    }


}

pub fn handler(ctx:Context<Take>)->Result<()>{
    ctx.accounts.deposit()?;
    ctx.accounts.withdraw_and_close_vault()?;
    Ok(())
}