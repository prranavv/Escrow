use anchor_lang::prelude::*;
use anchor_spl::token_2022::TransferChecked;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use anchor_spl::associated_token::AssociatedToken;
use crate::errors::EscrowError;
use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub maker:Signer<'info>,
    pub mint_a:InterfaceAccount<'info, Mint>,
    pub mint_b:InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_mint_a_ata:InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer=maker,
        space=8+Escrow::INIT_SPACE,
        seeds=[b"escrow",maker.key().as_ref(),seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow:Account<'info, Escrow>,
    #[account(
        init,
        payer=maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault:InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info,System>
}

impl <'info> Initialize<'info>{
    pub fn initialize_escrow(&mut self,recieve:u64,seed:u64,bump:u8)->Result<()>{
        self.escrow.set_inner(Escrow { 
            maker:*self.maker.key , 
            mint_a: self.mint_a.key(),
            mint_b:self.mint_b.key(), 
            maker_mint_a_ata: self.maker_mint_a_ata.key(), 
            vault: self.vault.key(), 
            bump, 
            recieve, 
            seed 
        });
        Ok(())
    }

    pub fn deposit_to_vault(&mut self,amount:u64)->Result<()>{
        let decimals= self.mint_a.decimals;
        let cpi_accounts = TransferChecked{
            mint:self.mint_a.to_account_info(),
            from:self.maker_mint_a_ata.to_account_info(),
            to:self.vault.to_account_info(),
            authority:self.maker.to_account_info()
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token_interface::transfer_checked(cpi_context, amount, decimals)?;
        Ok(())
    }
}

pub fn handler(ctx:Context<Initialize>,recieve:u64,seed:u64,amount:u64)->Result<()>{
    require!(recieve>0,EscrowError::InvalidRecieve);
    require!(amount>0,EscrowError::InvalidAmount);
    
    let bump = ctx.bumps.escrow;
    ctx.accounts.initialize_escrow(recieve,seed,bump)?;
    msg!("Escrow Initialized");
    ctx.accounts.deposit_to_vault(amount)?;
    msg!("Amount deposited to vault");
    Ok(())
}