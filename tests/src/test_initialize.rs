use litesvm::LiteSVM;
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_sdk::{message::{AccountMeta, Instruction}, pubkey::Pubkey, signature::{Keypair, read_keypair_file}, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_token::native_mint::DECIMALS;
use solana_transaction::{Transaction};

#[derive(Debug)]
struct InitializeArgs{
    pub recieve:u64,
    pub amount:u64,
    pub seed:u64
}

#[test]
fn test_initialize_and_take(){
    let mut svm = LiteSVM::new();
    let program_keypair = read_keypair_file("../target/deploy/escrow-keypair.json").unwrap();
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../../target/deploy/escrow.so");

    svm.add_program(program_id, program_bytes).unwrap();

    let maker = Keypair::new();
    let taker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&taker.pubkey(), 10_000_000_000).unwrap();

    let mint_a = CreateMint::new(&mut svm, &maker)
                                    .authority(&maker.pubkey())
                                    .decimals(DECIMALS)
                                    .send()
                                    .unwrap();

    let mint_b = CreateMint::new(&mut svm, &maker)
                                    .authority(&maker.pubkey())
                                    .decimals(DECIMALS)
                                    .send()
                                    .unwrap();

    let maker_mint_a_ata = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
        .owner(&maker.pubkey())
        .send()
        .unwrap();

    let taker_mint_b_ata = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    let maker_mint_b_ata = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_b)
        .owner(&maker.pubkey())
        .send()
        .unwrap();

    let taker_mint_a_ata = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut svm, &maker, &mint_a, &maker_mint_a_ata, 1_000_000_000)
        .owner(&maker)
        .send()
        .unwrap();

    MintTo::new(&mut svm, &taker, &mint_b, &taker_mint_b_ata, 500_000_000)
        .owner(&maker)
        .send()
        .unwrap();

    //test initialize

    let seed:u64 =10;
    let (escrow_pda,_pda) = Pubkey::find_program_address(
        &[b"escrow",maker.pubkey().as_ref(),&seed.to_le_bytes()],
         &program_id);
    
    let vault = get_associated_token_address(&escrow_pda, &mint_a);

    let discriminator = [175,
        175,
        109,
        31,
        13,
        152,
        155,
        237];
    let mut make_discriminator = [0u8; 8];

    make_discriminator.copy_from_slice(&discriminator[..8]);

    let initializeargs = InitializeArgs{
        recieve:500_000_000,
        amount:1_000_000_000,
        seed
    };

    let mut initialize_instruction_data = make_discriminator.to_vec();
    initialize_instruction_data.extend_from_slice(&initializeargs.seed.to_le_bytes());
    initialize_instruction_data.extend_from_slice(&initializeargs.recieve.to_le_bytes());
    initialize_instruction_data.extend_from_slice(&initializeargs.amount.to_le_bytes());

    let initialize_instruction = Instruction{
        program_id,
        accounts:vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new_readonly(mint_a, false),
            AccountMeta::new_readonly(mint_b, false),
            AccountMeta::new(maker_mint_a_ata, false),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new(vault,false),
            AccountMeta::new_readonly(spl_associated_token_account::id(),false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false)
        ],
        data:initialize_instruction_data
    };

    let initialize_tx = Transaction::new_signed_with_payer(&[initialize_instruction], Some(&maker.pubkey()), &[&maker], svm.latest_blockhash());
    svm.send_transaction(initialize_tx).unwrap();

    let maker_mint_a_ata_state =litesvm_token::get_spl_account::<spl_token::state::Account>(&svm, &maker_mint_a_ata).unwrap();
    assert_eq!(maker_mint_a_ata_state.amount,0,"Maker should have deposited the amount to the vault");

    let vault_state=litesvm_token::get_spl_account::<spl_token::state::Account>(&svm, &vault).unwrap();
    assert_eq!(vault_state.amount,1_000_000_000);
    //take instruction

    let take_discriminator_idl=[
        149,
        226,
        52,
        104,
        6,
        142,
        230,
        39
    ];

    let mut take_discriminator = [0u8; 8];
    take_discriminator.copy_from_slice(&take_discriminator_idl[..8]);

    let take_instruction_data=take_discriminator.to_vec();

    let take_instruction = Instruction{
    program_id,
    accounts:vec![
        AccountMeta::new(taker.pubkey(), true),
        AccountMeta::new(maker.pubkey(),false),
        AccountMeta::new_readonly(mint_a, false),
        AccountMeta::new_readonly(mint_b, false),
        AccountMeta::new(taker_mint_a_ata, false),
        AccountMeta::new(taker_mint_b_ata,false),
        AccountMeta::new(maker_mint_b_ata,false),
        AccountMeta::new(escrow_pda,false),
        AccountMeta::new(vault,false),
        AccountMeta::new_readonly(spl_associated_token_account::id(),false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_system_interface::program::ID, false)
    ],
    data:take_instruction_data
    };

    let taker_tx = Transaction::new_signed_with_payer(&[take_instruction], Some(&taker.pubkey()), &[&taker], svm.latest_blockhash());
    svm.send_transaction(taker_tx).unwrap();

    let escrow_closed =match svm.get_account(&escrow_pda){
        None=>true,
        Some(a)=>a.lamports==0 && a.data.is_empty(),
    };
    assert!(escrow_closed,"Escrow should be closed");

    let vault_closed = match svm.get_account(&vault){
        None=>true,
        Some(a)=>a.lamports==0&& a.data.is_empty(),
    };
    assert!(vault_closed,"Vault should be closed");

    let taker_mint_a_ata_state = litesvm_token::get_spl_account::<spl_token::state::Account>(&svm, &taker_mint_a_ata).unwrap();
    assert_eq!(taker_mint_a_ata_state.amount,1_000_000_000,"Taker should have recieved");

    let taker_mint_b_ata_state=litesvm_token::get_spl_account::<spl_token::state::Account>(&svm, &taker_mint_b_ata).unwrap();
    assert_eq!(taker_mint_b_ata_state.amount,0,"Taker should all mint b");

    let maker_mint_b_ata_state = litesvm_token::get_spl_account::<spl_token::state::Account>(&svm, &maker_mint_b_ata).unwrap();
    assert_eq!(maker_mint_b_ata_state.amount,500_000_000,"Maker should have recieved this amount")
}