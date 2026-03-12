# Escrow

A Solana escrow program built with [Anchor](https://www.anchor-lang.com/), enabling trustless token swaps between two parties using the SPL Token 2022 interface.

## How It Works

A **maker** creates an escrow by depositing Token A into a program-controlled vault and specifying how much of Token B they want in return. A **taker** can then fulfill the escrow by sending the requested Token B to the maker, receiving the deposited Token A from the vault. If no one takes the offer, the maker can reclaim their tokens through a refund.

## Instructions

### Initialize

Creates a new escrow and deposits tokens into the vault.

- **Params:** `receive` (amount of Token B expected), `amount` (Token A to deposit), `seed` (unique identifier)
- Validates that both `receive` and `amount` are greater than zero
- Derives the escrow PDA from `["escrow", maker, seed]`

### Take

Fulfills the escrow — the taker sends Token B to the maker and receives Token A from the vault.

- Transfers the requested Token B from taker to maker
- Transfers all Token A from the vault to the taker
- Closes the vault and the escrow account, returning rent to the maker

### Refund

Allows the maker to cancel the escrow and reclaim deposited tokens.

- Transfers Token A from the vault back to the maker
- Closes the vault and the escrow account

## Project Structure

```
├── lib.rs            # Program entrypoint and instruction routing
├── instructions/
│   ├── mod.rs        # Module exports
│   ├── initialize.rs # Escrow creation and token deposit
│   ├── take.rs       # Escrow fulfillment
│   └── refund.rs     # Escrow cancellation
├── state/            # Escrow account definition
└── errors.rs         # Custom error types
```

## Build & Test

```bash
anchor build
anchor test
```

## Program ID

```
F787RkPTqsZe4ZJUijreGS42hKrznXrvkNJ3P7d97j7s
```