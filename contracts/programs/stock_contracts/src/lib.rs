use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, MintTo, Burn, Transfer};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("9MWyubXRFZawmGVE9WqQXCvQnS1YiRx3u35vkeKaNbrL");

#[program]
pub mod stock_contracts {
    use super::*;

    pub fn initialize_trading_pool(
        ctx: Context<InitializeTradingPool>,
        vault_authority: Pubkey,
        backend_authority: Pubkey,
    ) -> Result<()> {
        let trading_pool = &mut ctx.accounts.trading_pool;
        trading_pool.vault_authority = vault_authority;
        trading_pool.backend_authority = backend_authority;
        trading_pool.total_orders = 0;
        trading_pool.bump = ctx.bumps.trading_pool;
        
        Ok(())
    }

    pub fn create_stock_mint(
        ctx: Context<CreateStockMint>,
        stock_symbol: String,
        decimals: u8,
    ) -> Result<()> {
        require!(stock_symbol.len() <= 10, StockTradingError::StockSymbolTooLong);
        
        let stock_mint_info = &mut ctx.accounts.stock_mint_info;
        stock_mint_info.stock_symbol = stock_symbol.clone();
        stock_mint_info.mint = ctx.accounts.stock_mint.key();
        stock_mint_info.total_supply = 0;
        stock_mint_info.bump = ctx.bumps.stock_mint_info;

        emit!(StockMintCreated {
            stock_symbol,
            mint: ctx.accounts.stock_mint.key(),
            decimals,
        });

        Ok(())
    }

    pub fn place_buy_order(
        ctx: Context<PlaceBuyOrder>,
        stock_symbol: String,
        sol_amount: u64,
        max_price_per_share: u64,
    ) -> Result<()> {
        require!(stock_symbol.len() <= 10, StockTradingError::StockSymbolTooLong);
        require!(sol_amount > 0, StockTradingError::InvalidAmount);

        let trading_pool = &mut ctx.accounts.trading_pool;
        let buy_order = &mut ctx.accounts.buy_order;

        // Transfer SOL from user to trading pool vault
        let transfer_instruction = anchor_lang::system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.trading_pool_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
        );
        anchor_lang::system_program::transfer(cpi_ctx, sol_amount)?;

        // Initialize buy order
        buy_order.user = ctx.accounts.user.key();
        buy_order.stock_symbol = stock_symbol.clone();
        buy_order.sol_amount = sol_amount;
        buy_order.max_price_per_share = max_price_per_share;
        buy_order.order_id = trading_pool.total_orders;
        buy_order.status = OrderStatus::Pending;
        buy_order.timestamp = Clock::get()?.unix_timestamp;
        buy_order.shares_received = 0;
        buy_order.actual_price_per_share = 0;
        buy_order.bump = ctx.bumps.buy_order;

        trading_pool.total_orders += 1;

        emit!(BuyOrderPlaced {
            order_id: buy_order.order_id,
            user: buy_order.user,
            stock_symbol: stock_symbol,
            sol_amount: sol_amount,
            max_price_per_share: max_price_per_share,
            timestamp: buy_order.timestamp,
        });

        Ok(())
    }

    pub fn fulfill_buy_order(
        ctx: Context<FulfillBuyOrder>,
        shares_purchased: u64,
        price_per_share: u64,
        total_cost: u64,
        refund_amount: u64,
    ) -> Result<()> {
        let buy_order = &mut ctx.accounts.buy_order;
        let trading_pool = &ctx.accounts.trading_pool;
        
        // Only backend authority can fulfill orders
        require!(
            ctx.accounts.backend_authority.key() == trading_pool.backend_authority,
            StockTradingError::UnauthorizedBackend
        );

        require!(
            buy_order.status == OrderStatus::Pending,
            StockTradingError::InvalidOrderStatus
        );
        require!(
            price_per_share <= buy_order.max_price_per_share,
            StockTradingError::PriceExceedsLimit
        );
        require!(
            total_cost + refund_amount <= buy_order.sol_amount,
            StockTradingError::InvalidCalculation
        );

        // Mint stock tokens to user
        if shares_purchased > 0 {
            let seeds = &[
                b"trading_pool".as_ref(),
                &[trading_pool.bump],
            ];
            let signer = &[&seeds[..]];

            let cpi_accounts = MintTo {
                mint: ctx.accounts.stock_mint.to_account_info(),
                to: ctx.accounts.user_stock_token_account.to_account_info(),
                authority: ctx.accounts.trading_pool.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            
            token::mint_to(cpi_ctx, shares_purchased)?;

            // Update stock mint info
            let stock_mint_info = &mut ctx.accounts.stock_mint_info;
            stock_mint_info.total_supply = stock_mint_info.total_supply
                .checked_add(shares_purchased)
                .ok_or(StockTradingError::Overflow)?;
        }

        // Update order status
        buy_order.status = OrderStatus::Fulfilled;
        buy_order.shares_received = shares_purchased;
        buy_order.actual_price_per_share = price_per_share;

        // Refund excess SOL if any
        if refund_amount > 0 {
            let vault_bump = ctx.bumps.trading_pool_vault;
            let seeds = &[
                b"trading_pool_vault".as_ref(),
                &[vault_bump],
            ];
            let signer = &[&seeds[..]];

            let transfer_instruction = anchor_lang::system_program::Transfer {
                from: ctx.accounts.trading_pool_vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                transfer_instruction,
                signer,
            );
            anchor_lang::system_program::transfer(cpi_ctx, refund_amount)?;
        }

        emit!(BuyOrderFulfilled {
            order_id: buy_order.order_id,
            user: buy_order.user,
            stock_symbol: buy_order.stock_symbol.clone(),
            shares_purchased,
            price_per_share,
            total_cost,
            refund_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn place_sell_order(
        ctx: Context<PlaceSellOrder>,
        stock_symbol: String,
        shares_to_sell: u64,
        min_price_per_share: u64,
    ) -> Result<()> {
        require!(stock_symbol.len() <= 10, StockTradingError::StockSymbolTooLong);
        require!(shares_to_sell > 0, StockTradingError::InvalidAmount);

        let trading_pool = &mut ctx.accounts.trading_pool;
        let sell_order = &mut ctx.accounts.sell_order;

        // Check user has enough tokens
        require!(
            ctx.accounts.user_stock_token_account.amount >= shares_to_sell,
            StockTradingError::InsufficientTokens
        );

        // Transfer tokens from user to escrow (controlled by trading pool)
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_stock_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, shares_to_sell)?;

        // Initialize sell order
        sell_order.user = ctx.accounts.user.key();
        sell_order.stock_symbol = stock_symbol.clone();
        sell_order.shares_to_sell = shares_to_sell;
        sell_order.min_price_per_share = min_price_per_share;
        sell_order.order_id = trading_pool.total_orders;
        sell_order.status = OrderStatus::Pending;
        sell_order.timestamp = Clock::get()?.unix_timestamp;
        sell_order.sol_received = 0;
        sell_order.actual_price_per_share = 0;
        sell_order.bump = ctx.bumps.sell_order;

        trading_pool.total_orders += 1;

        emit!(SellOrderPlaced {
            order_id: sell_order.order_id,
            user: sell_order.user,
            stock_symbol: stock_symbol,
            shares_to_sell: shares_to_sell,
            min_price_per_share: min_price_per_share,
            timestamp: sell_order.timestamp,
        });

        Ok(())
    }

    pub fn fulfill_sell_order(
        ctx: Context<FulfillSellOrder>,
        shares_sold: u64,
        price_per_share: u64,
        total_proceeds: u64,
        shares_returned: u64,
    ) -> Result<()> {
        let sell_order = &mut ctx.accounts.sell_order;
        let trading_pool = &ctx.accounts.trading_pool;
        
        // Only backend authority can fulfill orders
        require!(
            ctx.accounts.backend_authority.key() == trading_pool.backend_authority,
            StockTradingError::UnauthorizedBackend
        );

        require!(
            sell_order.status == OrderStatus::Pending,
            StockTradingError::InvalidOrderStatus
        );
        require!(
            price_per_share >= sell_order.min_price_per_share,
            StockTradingError::PriceBelowMinimum
        );
        require!(
            shares_sold + shares_returned == sell_order.shares_to_sell,
            StockTradingError::InvalidCalculation
        );

        let seeds = &[
            b"trading_pool".as_ref(),
            &[trading_pool.bump],
        ];
        let signer = &[&seeds[..]];

        // Burn sold tokens from escrow
        if shares_sold > 0 {
            let cpi_accounts = Burn {
                mint: ctx.accounts.stock_mint.to_account_info(),
                from: ctx.accounts.escrow_token_account.to_account_info(),
                authority: ctx.accounts.trading_pool.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            
            token::burn(cpi_ctx, shares_sold)?;

            // Update stock mint info
            let stock_mint_info = &mut ctx.accounts.stock_mint_info;
            stock_mint_info.total_supply = stock_mint_info.total_supply
                .checked_sub(shares_sold)
                .ok_or(StockTradingError::Underflow)?;

            // Transfer SOL proceeds to user
            let vault_bump = ctx.bumps.trading_pool_vault;
            let vault_seeds = &[
                b"trading_pool_vault".as_ref(),
                &[vault_bump],
            ];
            let vault_signer = &[&vault_seeds[..]];

            let transfer_instruction = anchor_lang::system_program::Transfer {
                from: ctx.accounts.trading_pool_vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                transfer_instruction,
                vault_signer,
            );
            anchor_lang::system_program::transfer(cpi_ctx, total_proceeds)?;
        }

        // Return unsold tokens to user
        if shares_returned > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.user_stock_token_account.to_account_info(),
                authority: ctx.accounts.trading_pool.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            
            token::transfer(cpi_ctx, shares_returned)?;
        }

        // Update order status
        sell_order.status = OrderStatus::Fulfilled;
        sell_order.sol_received = total_proceeds;
        sell_order.actual_price_per_share = price_per_share;

        emit!(SellOrderFulfilled {
            order_id: sell_order.order_id,
            user: sell_order.user,
            stock_symbol: sell_order.stock_symbol.clone(),
            shares_sold,
            price_per_share,
            total_proceeds,
            shares_returned,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn withdraw_vault_funds(
        ctx: Context<WithdrawVaultFunds>,
        amount: u64,
    ) -> Result<()> {
        let trading_pool = &ctx.accounts.trading_pool;
        
        // Only vault authority can withdraw
        require!(
            ctx.accounts.vault_authority.key() == trading_pool.vault_authority,
            StockTradingError::UnauthorizedVaultAccess
        );

        let vault_bump = ctx.bumps.trading_pool_vault;
        let seeds = &[
            b"trading_pool_vault".as_ref(),
            &[vault_bump],
        ];
        let signer = &[&seeds[..]];

        let transfer_instruction = anchor_lang::system_program::Transfer {
            from: ctx.accounts.trading_pool_vault.to_account_info(),
            to: ctx.accounts.vault_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
            signer,
        );
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        emit!(VaultFundsWithdrawn {
            authority: ctx.accounts.vault_authority.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn deposit_vault_funds(
        ctx: Context<DepositVaultFunds>,
        amount: u64,
    ) -> Result<()> {
        let transfer_instruction = anchor_lang::system_program::Transfer {
            from: ctx.accounts.vault_authority.to_account_info(),
            to: ctx.accounts.trading_pool_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
        );
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        emit!(VaultFundsDeposited {
            authority: ctx.accounts.vault_authority.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn update_authorities(
        ctx: Context<UpdateAuthorities>,
        new_vault_authority: Option<Pubkey>,
        new_backend_authority: Option<Pubkey>,
    ) -> Result<()> {
        let trading_pool = &mut ctx.accounts.trading_pool;
        
        // Only current vault authority can update authorities
        require!(
            ctx.accounts.current_vault_authority.key() == trading_pool.vault_authority,
            StockTradingError::UnauthorizedVaultAccess
        );

        if let Some(new_vault_auth) = new_vault_authority {
            trading_pool.vault_authority = new_vault_auth;
        }

        if let Some(new_backend_auth) = new_backend_authority {
            trading_pool.backend_authority = new_backend_auth;
        }

        emit!(AuthoritiesUpdated {
            vault_authority: trading_pool.vault_authority,
            backend_authority: trading_pool.backend_authority,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// Context structs
#[derive(Accounts)]
pub struct InitializeTradingPool<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + TradingPool::LEN,
        seeds = [b"trading_pool"],
        bump
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    /// CHECK: This is the trading pool vault that holds SOL
    #[account(
        seeds = [b"trading_pool_vault"],
        bump
    )]
    pub trading_pool_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(stock_symbol: String, _decimals: u8)]
pub struct CreateStockMint<'info> {
    #[account(
        init,
        payer = vault_authority,
        mint::decimals = 0,
        mint::authority = trading_pool,
        seeds = [b"stock_mint", stock_symbol.as_bytes()],
        bump
    )]
    pub stock_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = vault_authority,
        space = 8 + StockMintInfo::LEN,
        seeds = [b"stock_mint_info", stock_symbol.as_bytes()],
        bump
    )]
    pub stock_mint_info: Account<'info, StockMintInfo>,
    
    #[account(
        seeds = [b"trading_pool"],
        bump = trading_pool.bump,
        has_one = vault_authority
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    #[account(mut)]
    pub vault_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(stock_symbol: String)]
pub struct PlaceBuyOrder<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + BuyOrder::LEN,
        seeds = [
            b"buy_order",
            user.key().as_ref(),
            trading_pool.total_orders.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub buy_order: Account<'info, BuyOrder>,
    
    #[account(
        mut,
        seeds = [b"trading_pool"],
        bump = trading_pool.bump
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    /// CHECK: This is the trading pool vault that receives SOL
    #[account(
        mut,
        seeds = [b"trading_pool_vault"],
        bump
    )]
    pub trading_pool_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FulfillBuyOrder<'info> {
    #[account(
        mut,
        seeds = [
            b"buy_order",
            buy_order.user.as_ref(),
            buy_order.order_id.to_le_bytes().as_ref()
        ],
        bump = buy_order.bump
    )]
    pub buy_order: Account<'info, BuyOrder>,
    
    #[account(
        mut,
        seeds = [b"stock_mint", buy_order.stock_symbol.as_bytes()],
        bump
    )]
    pub stock_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"stock_mint_info", buy_order.stock_symbol.as_bytes()],
        bump = stock_mint_info.bump
    )]
    pub stock_mint_info: Account<'info, StockMintInfo>,
    
    #[account(
        init_if_needed,
        payer = backend_authority,
        associated_token::mint = stock_mint,
        associated_token::authority = user
    )]
    pub user_stock_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"trading_pool"],
        bump = trading_pool.bump
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    /// CHECK: This is the trading pool vault
    #[account(
        mut,
        seeds = [b"trading_pool_vault"],
        bump
    )]
    pub trading_pool_vault: AccountInfo<'info>,
    
    /// CHECK: User account to receive refund
    #[account(mut)]
    pub user: AccountInfo<'info>,
    
    #[account(mut)]
    pub backend_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(stock_symbol: String)]
pub struct PlaceSellOrder<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + SellOrder::LEN,
        seeds = [
            b"sell_order",
            user.key().as_ref(),
            trading_pool.total_orders.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub sell_order: Account<'info, SellOrder>,
    
    #[account(
        seeds = [b"stock_mint", stock_symbol.as_bytes()],
        bump
    )]
    pub stock_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = stock_mint,
        associated_token::authority = user
    )]
    pub user_stock_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        token::mint = stock_mint,
        token::authority = trading_pool,
        seeds = [b"escrow", stock_mint.key().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"trading_pool"],
        bump = trading_pool.bump
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FulfillSellOrder<'info> {
    #[account(
        mut,
        seeds = [
            b"sell_order",
            sell_order.user.as_ref(),
            sell_order.order_id.to_le_bytes().as_ref()
        ],
        bump = sell_order.bump
    )]
    pub sell_order: Account<'info, SellOrder>,
    
    #[account(
        mut,
        seeds = [b"stock_mint", sell_order.stock_symbol.as_bytes()],
        bump
    )]
    pub stock_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"stock_mint_info", sell_order.stock_symbol.as_bytes()],
        bump = stock_mint_info.bump
    )]
    pub stock_mint_info: Account<'info, StockMintInfo>,
    
    #[account(
        mut,
        associated_token::mint = stock_mint,
        associated_token::authority = sell_order.user
    )]
    pub user_stock_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = stock_mint,
        token::authority = trading_pool,
        seeds = [b"escrow", stock_mint.key().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"trading_pool"],
        bump = trading_pool.bump
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    /// CHECK: This is the trading pool vault
    #[account(
        mut,
        seeds = [b"trading_pool_vault"],
        bump
    )]
    pub trading_pool_vault: AccountInfo<'info>,
    
    /// CHECK: User account to receive SOL
    #[account(mut)]
    pub user: AccountInfo<'info>,
    
    #[account(mut)]
    pub backend_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawVaultFunds<'info> {
    #[account(
        seeds = [b"trading_pool"],
        bump = trading_pool.bump,
        has_one = vault_authority
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    /// CHECK: This is the trading pool vault
    #[account(
        mut,
        seeds = [b"trading_pool_vault"],
        bump
    )]
    pub trading_pool_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub vault_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositVaultFunds<'info> {
    #[account(
        seeds = [b"trading_pool"],
        bump = trading_pool.bump,
        has_one = vault_authority
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    /// CHECK: This is the trading pool vault
    #[account(
        mut,
        seeds = [b"trading_pool_vault"],
        bump
    )]
    pub trading_pool_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub vault_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAuthorities<'info> {
    #[account(
        mut,
        seeds = [b"trading_pool"],
        bump = trading_pool.bump,
        has_one = vault_authority @ StockTradingError::UnauthorizedVaultAccess
    )]
    pub trading_pool: Account<'info, TradingPool>,
    
    pub current_vault_authority: Signer<'info>,
    /// CHECK: Alias for current_vault_authority to satisfy has_one constraint
    pub vault_authority: UncheckedAccount<'info>,
}

// Account structs
#[account]
pub struct TradingPool {
    pub vault_authority: Pubkey,
    pub backend_authority: Pubkey,
    pub total_orders: u64,
    pub bump: u8,
}

impl TradingPool {
    pub const LEN: usize = 32 + 32 + 8 + 1;
}

#[account]
pub struct StockMintInfo {
    pub stock_symbol: String,
    pub mint: Pubkey,
    pub total_supply: u64,
    pub bump: u8,
}

impl StockMintInfo {
    pub const LEN: usize = (4 + 10) + 32 + 8 + 1;
}

#[account]
pub struct BuyOrder {
    pub user: Pubkey,
    pub stock_symbol: String,
    pub sol_amount: u64,
    pub max_price_per_share: u64,
    pub order_id: u64,
    pub status: OrderStatus,
    pub timestamp: i64,
    pub shares_received: u64,
    pub actual_price_per_share: u64,
    pub bump: u8,
}

impl BuyOrder {
    pub const LEN: usize = 32 + (4 + 10) + 8 + 8 + 8 + 1 + 8 + 8 + 8 + 1;
}

#[account]
pub struct SellOrder {
    pub user: Pubkey,
    pub stock_symbol: String,
    pub shares_to_sell: u64,
    pub min_price_per_share: u64,
    pub order_id: u64,
    pub status: OrderStatus,
    pub timestamp: i64,
    pub sol_received: u64,
    pub actual_price_per_share: u64,
    pub bump: u8,
}

impl SellOrder {
    pub const LEN: usize = 32 + (4 + 10) + 8 + 8 + 8 + 1 + 8 + 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Fulfilled,
    Cancelled,
}

// Events
#[event]
pub struct StockMintCreated {
    pub stock_symbol: String,
    pub mint: Pubkey,
    pub decimals: u8,
}

#[event]
pub struct BuyOrderPlaced {
    pub order_id: u64,
    pub user: Pubkey,
    pub stock_symbol: String,
    pub sol_amount: u64,
    pub max_price_per_share: u64,
    pub timestamp: i64,
}

#[event]
pub struct BuyOrderFulfilled {
    pub order_id: u64,
    pub user: Pubkey,
    pub stock_symbol: String,
    pub shares_purchased: u64,
    pub price_per_share: u64,
    pub total_cost: u64,
    pub refund_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct SellOrderPlaced {
    pub order_id: u64,
    pub user: Pubkey,
    pub stock_symbol: String,
    pub shares_to_sell: u64,
    pub min_price_per_share: u64,
    pub timestamp: i64,
}

#[event]
pub struct SellOrderFulfilled {
    pub order_id: u64,
    pub user: Pubkey,
    pub stock_symbol: String,
    pub shares_sold: u64,
    pub price_per_share: u64,
    pub total_proceeds: u64,
    pub shares_returned: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultFundsWithdrawn {
    pub authority: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultFundsDeposited {
    pub authority: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct AuthoritiesUpdated {
    pub vault_authority: Pubkey,
    pub backend_authority: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum StockTradingError {
    #[msg("Stock symbol too long")]
    StockSymbolTooLong,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid order status")]
    InvalidOrderStatus,
    #[msg("Price exceeds maximum limit")]
    PriceExceedsLimit,
    #[msg("Price below minimum limit")]
    PriceBelowMinimum,
    #[msg("Invalid calculation")]
    InvalidCalculation,
    #[msg("Unauthorized backend access")]
    UnauthorizedBackend,
    #[msg("Unauthorized vault access")]
    UnauthorizedVaultAccess,
    #[msg("Insufficient tokens")]
    InsufficientTokens,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
}
