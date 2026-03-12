use anchor_client::solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_litesvm::AnchorLiteSVM;



declare_id!("F787RkPTqsZe4ZJUijreGS42hKrznXrvkNJ3P7d97j7s");

#[test]
fn test_initialize(){
    let ctx = AnchorLiteSVM::new()
        .deploy_program(id(), include_bytes!("../../target/deploy/escrow.so"))
        .build();

    let maker= ctx.create_funded_account(10*LAMPORTS_PER_SOL).unwrap();
    let payer = ctx.create_funded_account(10*LAMPORTS_PER_SOL).unwrap();
}