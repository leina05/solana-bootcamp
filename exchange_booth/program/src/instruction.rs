use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ExchangeBoothInstruction {
    /// Initializes an Exchange Booth (EB) for a given token pair, admin, and oracle.
    /// An EB admin has the ability to withdraw from the EB vault accounts and close the EB.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` EB admin account.
    ///   1. `[]` Mint account of the base currency.
    ///   2. `[]` Mint account of the quote currency.
    ///   3. `[]` Oracle program account.
    ///   4. `[]` Token Program.
    ///   5. `[]` System Program.
    ///   6. `[]` Rent Sysvar.
    ///   7. `[]` EB state account (PDA).
    ///   8. `[]` Vault token account of the base currency (PDA).
    ///   9. `[]` Vault token account of the quote currency (PDA).
    InititializeExchangeBooth {
        vault_base_bump: u8,
        vault_quote_bump: u8,
        state_bump: u8,
    },
    /// Transfers tokens from an admin-owned token account to an EB vault.
    ///
    ///   0. `[signer]` EB admin account.
    ///   1. `[writable]` Token account owned by EB admin.
    ///   2. `[]` Token Program.
    ///   3. `[writable]` Vault token account of the deposit currency (PDA).
    Deposit {
        /// Mint account of the deposit token.
        mint: Pubkey,
        /// Amount of token to deposit (before decimals).
        /// E.g., float amount = amount * 10e(-decimals)
        amount: u64,
    },
    /// Withdraws tokens from an EB vault to an admin-owned token account.
    ///
    ///   0. `[signer]` EB admin account.
    ///   1. `[writable]` Token account owned by EB admin.
    ///   2. `[]` Token Program.
    ///   3. `[writable]` Vault token account of the withdrawal currency (PDA).
    Withdraw {
        /// Mint account of the deposit token.
        mint: Pubkey,
        /// Amount of token to withdraw (before decimals).
        /// E.g., float amount = amount * 10e(-decimals)
        amount: u64,
    },
    /// Exchanges an amount of tokens in one currency for the corresponding amount in another currency.
    /// Exchange rate is determined by the oracle.
    /// Exchanged tokens are depoosited directly into the user's token account.
    ///
    ///   0. `[signer]` EB user account.
    ///   1. `[writable]` Token account for input currency owned by EB user.
    ///   2. `[writable]` Token account for output currency owned by EB user.
    ///   3. `[]` Oracle program account.
    ///   4. `[]` Token Program.
    ///   5. `[]` Vault token account of the base currency (PDA).
    ///   6. `[]` Vault token account of the quote currency (PDA).
    ///   7. `[]` EB state account (PDA).
    Exchange {
        /// Mint account of the input token.
        input_mint: Pubkey, // TODO
        /// Amount of input token to exchange (before decimals).
        /// E.g., float amount = amount * 10e(-decimals)
        amount: u64,
    },
    /// Closes an EB for a given admin, currency pair, and oracle.
    ///
    ///   0. `[signer]` EB admin account.
    ///   1. `[writable]` Token account for base currenccy owned by EB admin.
    ///   2. `[writable]` Token account for quote currenccy owned by EB admin.
    ///   3. `[]` Token Program.
    ///   4. `[]` Vault token account of the base currency (PDA).
    ///   5. `[]` Vault token account of the quote currency (PDA).
    ///   6. `[]` EB state account (PDA).
    CloseExchangeBooth,
}
