use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_cloud_funding {
    use super::*;

    pub fn create(ctx: Context<Create>, name: String, description: String) -> ProgramResult {
        // ここで受け取るctx.accounts.campaignはPDAになっている
        let campaign = &mut ctx.accounts.campaign;
        campaign.name = name;
        campaign.description = description;
        campaign.amount_donated = 0;
        campaign.admin = *ctx.accounts.user.key;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> ProgramResult {
        let campaign = &mut ctx.accounts.campaign;
        let user = &mut ctx.accounts.user; // walletのpubkey
        if campaign.admin != *user.key {
            return Err(ProgramError::IncorrectProgramId);
        }

        // campaign accountを管理するのに必要な最低限のlamport
        // https://docs.rs/solana-program/1.10.29/solana_program/rent/struct.Rent.html#method.minimum_balance
        let rent_balance = Rent::get()?.minimum_balance(campaign.to_account_info().data_len());
        if **campaign.to_account_info().lamports.borrow() - rent_balance < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        // PDAから別のwalletに通貨を送るときはtry_borrow_mut_lamportsを使用しなければいけない
        // https://discord.com/channels/889577356681945098/889702325231427584/999690159920521256
        **campaign.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;

        (&mut ctx.accounts.campaign).amount_donated -= amount;
        Ok(())
    }

    pub fn donate(ctx: Context<Donate>, amount: u64) -> ProgramResult {
        // お金を送るinstruction作成
        // 誰から誰にいくら渡すか設定する
        // https://docs.rs/solana-program/1.10.29/solana_program/system_instruction/fn.transfer.html
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.campaign.key(),
            amount
        );

        // これでtransactionを実行している、これはprogramから別のprogramを呼び出すためのもの
        // 今回のdonateからtransferを呼び出している
        // 第一引数にinstruction
        // 第二引数にtransferで必要なaccount情報を含ませる
        // https://docs.rs/solana-program/1.10.29/solana_program/program/fn.invoke.html
        match anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.campaign.to_account_info()
            ]
        ) {
            Ok(result) => result,
            Err(error) => panic!("function donate: {:?}", error),
        };

        // campaignアカウントにお金を追加。
        // campaignアカウントはPDAで権限はこのsmart contractにあるので可能
        (&mut ctx.accounts.campaign).amount_donated += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer=user, space=9000, seeds=[b"CAMPAIGN_DEMO".as_ref(), user.key().as_ref()], bump)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    pub campaign: Account<'info, Campaign>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[account]
pub struct Campaign {
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
    pub amount_donated: u64
}