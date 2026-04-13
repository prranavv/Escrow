<h1 align="center">Escrow</h1>
<p align="center"><strong>Trustless Token Swaps on Solana</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75+-DEA584?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/Solana-2.0-9945FF?style=flat-square&logo=solana" />
  <img src="https://img.shields.io/badge/Anchor-1.0-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/Token--2022-✓-00D18C?style=flat-square" />
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" />
</p>

<p align="center">
  A Solana escrow program built with Anchor, enabling trustless token swaps between two parties using the SPL Token 2022 interface. A maker deposits tokens and sets their terms — a taker either fulfills the swap or the maker reclaims their deposit. No intermediaries, no trust required.
</p>

---

## The Problem

Peer-to-peer token swaps require trust — if Alice sends her tokens first, Bob could simply walk away. Without a neutral third party or an on-chain mechanism to enforce atomicity, one side always bears the counterparty risk.

## The Solution

This escrow program acts as a trustless intermediary on Solana. A **maker** deposits Token A into a program-controlled vault and specifies how much Token B they want in return. A **taker** can fulfill the escrow atomically — Token B goes to the maker, Token A goes to the taker — in a single transaction. If no one takes the offer, the maker can reclaim their tokens through a refund. The program enforces that both sides of the swap happen together or not at all.

**Program ID:** `F787RkPTqsZe4ZJUijreGS42hKrznXrvkNJ3P7d97j7s`

---

## How It Works

```
┌──────────────────────────────────────────────────────────────┐
│                        Initialize                            │
│                                                              │
│   Maker ──── Token A ────►  Vault (PDA)                      │
│              deposits        program-controlled              │
│              amount          escrow stores terms              │
│                              (receive amount, seed)          │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                           Take                               │
│                                                              │
│   Taker ──── Token B ────►  Maker                            │
│              sends                                           │
│              requested                                       │
│              amount                                          │
│                                                              │
│   Vault ──── Token A ────►  Taker                            │
│              releases                                        │
│              deposit                                         │
│                                                              │
│   Vault + Escrow account closed, rent returned to Maker      │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                          Refund                              │
│                                                              │
│   Vault ──── Token A ────►  Maker                            │
│              returns                                         │
│              deposit                                         │
│                                                              │
│   Vault + Escrow account closed, rent returned to Maker      │
└──────────────────────────────────────────────────────────────┘
```

---

## Features

- **Trustless swaps** — both sides of the exchange happen atomically in a single transaction, eliminating counterparty risk
- **Program-controlled vault** — deposited tokens are held in a PDA-owned token account that only the program can access
- **Maker-defined terms** — the maker specifies exactly how much Token B they want in return for their Token A deposit
- **Refundable deposits** — if no taker fulfills the escrow, the maker can reclaim their tokens at any time
- **Account cleanup** — both `take` and `refund` close the vault and escrow accounts, returning rent to the maker
- **Token-2022 support** — uses the `token_interface` for compatibility with both SPL Token and Token-2022 standards
- **Input validation** — rejects escrows where either the deposit or requested amount is zero

---

## Instructions

| Instruction | Parameters | Description |
|---|---|---|
| **`initialize`** | `receive: u64, amount: u64, seed: u64` | Creates a new escrow — deposits `amount` of Token A into a PDA vault and records that the maker wants `receive` of Token B. Derives escrow PDA from `["escrow", maker, seed]`. |
| **`take`** | — | Fulfills the escrow. Taker sends the requested Token B to the maker, receives all Token A from the vault. Closes the vault and escrow account, returning rent to the maker. |
| **`refund`** | — | Cancels the escrow. Returns deposited Token A from the vault back to the maker. Closes the vault and escrow account. |

---

## Account Structure

### Escrow (PDA)

| Field | Type | Description |
|---|---|---|
| `maker` | `Pubkey` | The maker's public key |
| `mint_a` | `Pubkey` | Mint address of the deposited token (Token A) |
| `mint_b` | `Pubkey` | Mint address of the requested token (Token B) |
| `receive` | `u64` | Amount of Token B the maker wants |
| `seed` | `u64` | Unique seed for PDA derivation |
| `bump` | `u8` | PDA bump |

### PDA Seeds

| Account | Seeds |
|---|---|
| **Escrow** | `["escrow", maker.key(), seed.to_le_bytes()]` |
| **Vault** | Associated token account owned by the escrow PDA |

---

## Project Structure

```
escrow/
├── src/
│   ├── lib.rs              # Program entrypoint and instruction routing
│   ├── instructions/
│   │   ├── mod.rs           # Module exports
│   │   ├── initialize.rs    # Escrow creation and token deposit
│   │   ├── take.rs          # Escrow fulfillment
│   │   └── refund.rs        # Escrow cancellation
│   ├── state/               # Escrow account definition
│   └── errors.rs            # Custom error types
├── Cargo.toml
└── README.md
```

---

## Quick Start

### Prerequisites

- Rust 1.75+
- Solana CLI 2.0+
- Anchor CLI 1.0+

### 1. Clone and build

```bash
git clone https://github.com/prranavv/Escro.git
cd Escro

anchor build
```

### 2. Deploy

```bash
anchor deploy
```

### 3. Test

```bash
anchor test
```

---

## Tech Stack

| Component | Technology | Purpose |
|---|---|---|
| **Runtime** | Solana | High-throughput L1 blockchain |
| **Framework** | Anchor 1.0 | Solana program development framework with IDL generation |
| **Token Standard** | SPL Token / Token-2022 | Fungible token operations via `token_interface` |
| **Language** | Rust | On-chain program logic |

---

## FAQ's

**"Why an escrow instead of a direct swap?"**
> A direct swap requires both parties to be online and sign the same transaction simultaneously. An escrow decouples the two sides — the maker deposits and walks away, and any taker can fulfill it later. This is the same pattern used by on-chain orderbooks and OTC desks.

**"Why is the vault a PDA-owned token account?"**
> PDA ownership means only the program can authorize transfers out of the vault. The maker can't pull their tokens back except through the `refund` instruction, and the taker can't drain the vault without sending the requested Token B first. This is what makes it trustless.

**"Why use a seed parameter?"**
> The seed is part of the escrow PDA derivation, so a single maker can have multiple active escrows simultaneously. Without it, each maker could only have one escrow at a time.

**"What happens to the rent when an escrow is fulfilled or refunded?"**
> Both `take` and `refund` close the vault token account and the escrow account, returning the rent-exempt SOL balance back to the maker.

---

## Disclaimer

This escrow program is a learning and portfolio project demonstrating Solana program development with the Anchor framework. It has not been audited and is not intended for production use with real funds. Always conduct a thorough security audit before deploying any program that handles user assets.

---

## License

MIT — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built by <a href="https://github.com/prranavv">prranavv</a></sub>
</p>