use anchor_spl::associated_token::get_associated_token_address;
use jupiter_amm_interface::{
    AccountMap,
    Amm,
    AmmContext,
    KeyedAccount,
    Quote,
    QuoteParams,
    Swap,
    SwapAndAccountMetas,
    SwapParams,
};
use anyhow::Result;
use solana_sdk::{ instruction::AccountMeta, system_program::ID as SystemProgramId };
use solana_pubkey::Pubkey;

pub mod constants;
use constants::*;

#[derive(Copy, Clone, Debug)]
pub struct BankinecoAmm {
    bank: Pubkey,
    vault: Pubkey,
    team: Pubkey,
    oracle: Pubkey,
    yielding_mint_program: Pubkey,
}

impl BankinecoAmm {
    pub fn new(vault: Pubkey) -> Self {
        let oracle = Pubkey::default(); // TODO
        let team = Pubkey::default(); // TODO
        Self {
            bank: USD_STAR_BANK,
            vault,
            team,
            oracle,
            yielding_mint_program: anchor_spl::token::ID,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BankinecoSwapAction {
    user: Pubkey,
    bank: Pubkey,
    vault: Pubkey,
    oracle: Pubkey,
    yielding_mint: Pubkey,
    bank_mint: Pubkey,
    team: Pubkey,
    system_program: Pubkey,
    token_program: Pubkey,
    yielding_mint_program: Pubkey,
    associated_token_program: Pubkey,
}

impl TryFrom<BankinecoSwapAction> for Vec<AccountMeta> {
    type Error = anyhow::Error;

    fn try_from(accounts: BankinecoSwapAction) -> Result<Self> {
        let yielding_user_ta = get_associated_token_address(
            &accounts.user,
            &accounts.yielding_mint
        );
        let bank_mint_user_ta = get_associated_token_address(&accounts.user, &accounts.bank_mint);
        let yielding_vault_ta = get_associated_token_address(
            &accounts.vault,
            &accounts.yielding_mint
        );
        let fee_team_ta = get_associated_token_address(&accounts.team, &accounts.yielding_mint);

        let account_metas = vec![
            AccountMeta::new(accounts.user, true),
            AccountMeta::new(accounts.bank, false),
            AccountMeta::new(accounts.vault, false),
            AccountMeta::new_readonly(accounts.oracle, false),
            AccountMeta::new(accounts.yielding_mint, false),
            AccountMeta::new(accounts.bank_mint, false),
            AccountMeta::new(yielding_user_ta, false),
            AccountMeta::new(bank_mint_user_ta, false),
            AccountMeta::new(yielding_vault_ta, false),
            AccountMeta::new(accounts.team, false),
            AccountMeta::new(fee_team_ta, false),
            AccountMeta::new_readonly(SystemProgramId, false),
            AccountMeta::new_readonly(anchor_spl::token::ID, false),
            AccountMeta::new_readonly(accounts.yielding_mint_program, false),
            AccountMeta::new_readonly(anchor_spl::associated_token::ID, false)
        ];

        Ok(account_metas)
    }
}

impl Amm for BankinecoAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount, _amm_context: &AmmContext) -> Result<Self>
        where Self: Sized
    {
        Ok(BankinecoAmm::new(keyed_account.key))
    }

    fn label(&self) -> String {
        "PerenaBankinecoAmm".to_string()
    }

    fn program_id(&self) -> Pubkey {
        PROGRAM_ID
    }

    fn key(&self) -> Pubkey {
        self.vault
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![USD_STAR_MINT, USDC_MINT]
    }

    /// The accounts necessary to produce a quote
    fn get_accounts_to_update(&self) -> Vec<Pubkey> {}

    /// Picks necessary accounts to update it's internal state
    /// Heavy deserialization and precomputation caching should be done in this function
    fn update(&mut self, account_map: &AccountMap) -> Result<()> {}

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {}

    /// Indicates which Swap has to be performed along with all the necessary account metas
    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams { source_mint, destination_mint, token_transfer_authority, .. } =
            swap_params;

        let user = token_transfer_authority;
        let is_mint = source_mint == &USDC_MINT;

        let (yielding_mint, bank_mint) = if is_mint {
            (source_mint, destination_mint)
        } else {
            (destination_mint, source_mint)
        };

        Ok(SwapAndAccountMetas {
            swap: Swap::TokenSwap,
            account_metas: (BankinecoSwapAction {
                user: *user,
                bank: self.bank,
                vault: self.vault,
                oracle: self.oracle,
                yielding_mint: *yielding_mint,
                bank_mint: *bank_mint,
                team: self.team,
                system_program: SystemProgramId,
                token_program: anchor_spl::token::ID,
                yielding_mint_program: self.yielding_mint_program,
                associated_token_program: anchor_spl::associated_token::ID,
            }).try_into()?,
        })
    }

    fn has_dynamic_accounts(&self) -> bool {
        false
    }

    /// Indicates whether `update` needs to be called before `get_reserve_mints`
    fn requires_update_for_reserve_mints(&self) -> bool {
        false
    }

    // Indicates that whether ExactOut mode is supported
    fn supports_exact_out(&self) -> bool {
        false
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }

    /// It can only trade in one direction from its first mint to second mint, assuming it is a two mint AMM
    fn unidirectional(&self) -> bool {
        false
    }

    /// For testing purposes, provide a mapping of dependency programs to function
    fn program_dependencies(&self) -> Vec<(Pubkey, String)> {
        vec![]
    }

    fn get_accounts_len(&self) -> usize {
        32 // Default to a near whole legacy transaction to penalize no implementation
    }

    /// Provides a shortcut to establish if the AMM can be used for trading
    /// If the market is active at all
    fn is_active(&self) -> bool {
        true
    }
}
