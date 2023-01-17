use anchor_lang::prelude::*;
use anchor_lang::solana_program::account_info::AccountInfo;
use anchor_lang::AccountsClose;
use std::string::*;

use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::{AssociatedToken, get_associated_token_address};

const BLANK: &str = "                                ";

declare_id!("HSmCq1hyEyUcrKqwEeW2nneDqfcC6ZP7iqfcyuJmgpn3");

#[program]
pub mod stream_contract {

    use super::*;

    pub fn create_stream(
        ctx: Context<CreateStream>,
        stream_id: String,
        stream_title: String,
        bump: u8,
        amount: u128,
        start: u128,
        interval: u128,
        rate: u128,
        duration: u128,
        is_infinite: bool,
        cancel_by: u8,
        pause_by: u8,
        resume_by: u8,
        withdraw_by: u8,
        edit_by: u8,
    ) -> Result<()> {
        // Get Account
        let stream_account = &mut ctx.accounts.stream;
        let stream_list_sender = &mut ctx.accounts.stream_list_sender;
        let stream_list_recipient = &mut ctx.accounts.stream_list_recipient;

        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;

        let next_stream_id = stream_list_sender.stream_id + 1;

        // Stream ID should match
        require!(
            stream_id == next_stream_id.to_string(),
            MyError::IncorrectStreamId
        );

        // Stream Title shouldn't be longer than 50 characters
        require!(stream_title.len() <= 50, MyError::TitleTooLong);

        // Recipient shouldn't be same as Sender
        require!(
            ctx.accounts.recipient.key() != ctx.accounts.sender.key(),
            MyError::SenderIsRecipient
        );

        // Amount to Stream should be greater than 0
        require!(amount > 0, MyError::DepositIsZero);

        // Start time of Stream should be in future
        require!(start >= timestamp, MyError::PastStartTime);

        // Interval of Stream should be greater than 0
        require!(interval > 0, MyError::IntervalIsZero);

        // Amount to Stream should be greater than the Rate of Stream
        require!(amount >= rate, MyError::DepositSmallerThanTime);

        require!(cancel_by <= 2, MyError::InvalidCancelBy);
        require!(pause_by <= 2, MyError::InvalidPauseBy);
        require!(withdraw_by <= 2, MyError::InvalidWithdrawBy);
        require!(resume_by <= 2, MyError::InvalidResumeBy);
        require!(edit_by <= 2, MyError::InvalidEditBy);

        require!(duration == ((amount as f32 / rate as f32) * interval as f32) as u128, MyError::IncorrectDuration);
        let rem = amount % rate;
        let no_of_intervals = amount/rate;

        let new_duration = match rem {
            1.. => interval * (no_of_intervals + 1),
            0 => duration
        };

        let stop = start + new_duration;

        stream_account.stream_id = stream_id.clone();
        stream_account.stream_title = stream_title;
        stream_account.recipient = ctx.accounts.recipient.key();
        stream_account.sender = ctx.accounts.sender.key();
        stream_account.token_address = Pubkey::new(BLANK.as_bytes());
        stream_account.start_time = start;
        stream_account.stop_time = stop;
        stream_account.remaining_balance = amount;
        stream_account.deposit = amount;
        stream_account.interval = interval;
        stream_account.rate_of_stream = rate;
        stream_account.bump = *ctx.bumps.get("stream").unwrap();
        stream_account.is_paused = false;
        stream_account.is_infinite = is_infinite;
        stream_account.is_cancelled = false;
        stream_account.cancel_by = match cancel_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.pause_by = match pause_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.withdraw_by = match withdraw_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.resume_by = match resume_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.edit_by = match edit_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };

        let sender = ctx.accounts.sender.key();

        let seeds = &[stream_id.as_bytes(), sender.as_ref(), &[bump]];

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.sender.key(),
            &stream_account.key(),
            amount as u64,
        );

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.sender.to_account_info(),
                stream_account.to_account_info(),
            ],
            &[&seeds[..]],
        )?;

        stream_list_sender.stream_id = next_stream_id;
        stream_list_sender.items.push(StreamItem {
            stream_list: ctx.accounts.stream.key(),
            is_sender: true,
        });

        stream_list_recipient.items.push(StreamItem {
            stream_list: ctx.accounts.stream.key(),
            is_sender: false,
        });

        Ok(())
    }

    pub fn create_stream_token(
        ctx: Context<CreateStreamToken>,
        stream_id: String,
        stream_title: String,
        values: Vec<u128>,
        is_infinite: bool,
        cancel_by: u8,
        pause_by: u8,
        resume_by: u8,
        withdraw_by: u8,
        edit_by: u8,
    ) -> Result<()> {
        let stream_account = &mut ctx.accounts.stream;
        let stream_list_sender = &mut ctx.accounts.stream_list_sender;
        let stream_list_recipient = &mut ctx.accounts.stream_list_recipient;

        let amount = values[0];
        let start = values[1];
        let interval = values[2];
        let rate = values[3];
        let duration = values[4];
        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;

        let next_stream_id = stream_list_sender.stream_id + 1;

        // Stream ID should match
        require!(
            stream_id == next_stream_id.to_string(),
            MyError::IncorrectStreamId
        );

        // Stream Title shouldn't be longer than 50 characters
        require!(stream_title.len() <= 50, MyError::TitleTooLong);

        // Recipient shouldn't be same as Sender
        require!(
            ctx.accounts.recipient.key() != ctx.accounts.sender.key(),
            MyError::SenderIsRecipient
        );

        // Amount to Stream should be greater than 0
        require!(amount > 0, MyError::DepositIsZero);

        // Start time of Stream should be in future
        require!(start >= timestamp, MyError::PastStartTime);

        // Interval of Stream should be greater than 0
        require!(interval > 0, MyError::IntervalIsZero);

        // Amount to Stream should be greater than the Rate of Stream
        require!(amount >= rate, MyError::DepositSmallerThanTime);
       
        require!(cancel_by <= 2, MyError::InvalidCancelBy);
        require!(pause_by <= 2, MyError::InvalidPauseBy);
        require!(withdraw_by <= 2, MyError::InvalidWithdrawBy);
        require!(resume_by <= 2, MyError::InvalidResumeBy);
        require!(edit_by <= 2, MyError::InvalidEditBy);
        
        require!(duration == ((amount as f32 / rate as f32).round() * interval as f32) as u128, MyError::IncorrectDuration);
        let rem = amount % rate;
        let no_of_intervals = amount/rate;

        let new_duration = match rem {
            1.. => interval * (no_of_intervals + 1),
            0 => duration
        };

        let stop = start + new_duration;

        stream_account.stream_id = stream_id;
        stream_account.stream_title = stream_title;
        stream_account.recipient = ctx.accounts.recipient.key();
        stream_account.sender = ctx.accounts.sender.key();
        stream_account.token_address = ctx.accounts.token_address.key();
        stream_account.start_time = start;
        stream_account.stop_time = stop;
        stream_account.remaining_balance = amount;
        stream_account.deposit = amount;
        stream_account.interval = interval;
        stream_account.rate_of_stream = rate;
        stream_account.bump = *ctx.bumps.get("stream").unwrap();
        stream_account.is_paused = false;
        stream_account.is_infinite = is_infinite;
        stream_account.is_cancelled = false;
        stream_account.cancel_by = match cancel_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.pause_by = match pause_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.withdraw_by = match withdraw_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.resume_by = match resume_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };
        stream_account.edit_by = match edit_by {
            0 => StateChangeAuth::OnlySender,
            1 => StateChangeAuth::OnlyReceiver,
            _ => StateChangeAuth::Both,
        };

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.sender_tokens.clone().to_account_info(),
                    to: ctx.accounts.stream_tokens.to_account_info(),
                    authority: ctx.accounts.sender.clone().to_account_info(),
                },
            ),
            amount as u64,
        )?;

        stream_list_sender.stream_id = next_stream_id;
        stream_list_sender.items.push(StreamItem {
            stream_list: ctx.accounts.stream.key(),
            is_sender: true,
        });

        stream_list_recipient.items.push(StreamItem {
            stream_list: ctx.accounts.stream.key(),
            is_sender: false,
        });

        Ok(())
    }

    pub fn withdraw_from_stream(ctx: Context<WithdrawFromStream>, stream_id: String) -> Result<()> {
        let stream_account = &mut ctx.accounts.stream;

        require!(
            (stream_account.sender == ctx.accounts.authority.key()
                && stream_account.withdraw_by != StateChangeAuth::OnlyReceiver)
                || (stream_account.recipient == ctx.accounts.authority.key()
                    && stream_account.withdraw_by != StateChangeAuth::OnlySender),
            MyError::NotAuthorized
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(stream_account.is_paused == false, MyError::StreamIsPaused);
        require!(
            ctx.accounts.recipient.key() == stream_account.recipient,
            MyError::IncorrectRecipient
        );
        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;
        let start = stream_account.start_time;
        let stop = stream_account.stop_time;
        let interval = stream_account.interval;

        require!(timestamp >= start, MyError::StreamNotStarted);

        let mut ready_for_withdrawal: u128;

        if timestamp >= stop {
            ready_for_withdrawal = stream_account.remaining_balance;
        } else {
            let delta = timestamp - start;
            require!(delta >= interval, MyError::NothingToWithdraw);

            let no_of_intervals = delta / interval;

            ready_for_withdrawal = no_of_intervals * stream_account.rate_of_stream;

            if stream_account.deposit > stream_account.remaining_balance {
                let amt_withdrawn = stream_account.deposit - stream_account.remaining_balance;
                ready_for_withdrawal -= amt_withdrawn;
            }
        }

        require!(ready_for_withdrawal > 0, MyError::NothingToWithdraw);

        let amt = ready_for_withdrawal as u64;

        **stream_account.to_account_info().try_borrow_mut_lamports()? -= amt;
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += amt;
        
        stream_account.remaining_balance -= amt as u128;

        if stream_account.remaining_balance == 0 {
            stream_account.is_cancelled = true;
        }
        Ok(())
    }

    pub fn withdraw_from_stream_token(
        ctx: Context<WithdrawFromStreamToken>,
        stream_id: String,
    ) -> Result<()> {
        let stream_account = &mut ctx.accounts.stream;

        require!(
            (stream_account.sender == ctx.accounts.authority.key()
                && stream_account.withdraw_by != StateChangeAuth::OnlyReceiver)
                || (stream_account.recipient == ctx.accounts.authority.key()
                    && stream_account.withdraw_by != StateChangeAuth::OnlySender),
            MyError::NotAuthorized
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(stream_account.is_paused == false, MyError::StreamIsPaused);
        require!(
            ctx.accounts.recipient.key() == stream_account.recipient,
            MyError::IncorrectRecipient
        );
        require!(
            ctx.accounts.token_address.key() == stream_account.token_address,
            MyError::IncorrectTokenAddress
        );
        require!(
            ctx.accounts.sender.key() == stream_account.sender,
            MyError::IncorrectSender
        );
        let recipient_tokens = get_associated_token_address(&ctx.accounts.recipient.key(), &stream_account.token_address);
        require!(ctx.accounts.recipient_tokens.key() == recipient_tokens, MyError::AssociatedTokenAccountIncorrect);

        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;
        let start = stream_account.start_time;
        let stop = stream_account.stop_time;
        let interval = stream_account.interval;

        require!(timestamp >= start, MyError::StreamNotStarted);

        let mut ready_for_withdrawal: u128;

        if timestamp >= stop {
            ready_for_withdrawal = stream_account.remaining_balance;
        } else {
            let delta = timestamp - start;
            require!(delta >= interval, MyError::NothingToWithdraw);

            let no_of_intervals = delta / interval;

            ready_for_withdrawal = no_of_intervals * stream_account.rate_of_stream;

            if stream_account.deposit > stream_account.remaining_balance {
                let amt_withdrawn = stream_account.deposit - stream_account.remaining_balance;
                ready_for_withdrawal -= amt_withdrawn;
            }
        }

        require!(ready_for_withdrawal > 0, MyError::NothingToWithdraw);

        let amt = ready_for_withdrawal as u64;

        let sender = ctx.accounts.sender.key();
        let bump = stream_account.bump;

        let seeds = &[stream_id.as_bytes(), sender.as_ref(), &[bump]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.stream_tokens.clone().to_account_info(),
                    to: ctx.accounts.recipient_tokens.to_account_info(),
                    authority: stream_account.to_account_info(),
                },
                &[&seeds[..]],
            ),
            amt,
        )?;

        stream_account.remaining_balance -= amt as u128;

        if stream_account.remaining_balance == 0 {
            stream_account.is_cancelled = true;
        }

        Ok(())
    }

    pub fn cancel_stream(ctx: Context<CancelStream>, stream_id: String) -> Result<()> {
        let stream_account = &mut ctx.accounts.stream;

        require!(
            (stream_account.sender == ctx.accounts.authority.key()
                && stream_account.cancel_by != StateChangeAuth::OnlyReceiver)
                || (stream_account.recipient == ctx.accounts.authority.key()
                    && stream_account.cancel_by != StateChangeAuth::OnlySender),
            MyError::NotAuthorized
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(
            ctx.accounts.recipient.key() == stream_account.recipient,
            MyError::IncorrectRecipient
        );
        require!(
            ctx.accounts.sender.key() == stream_account.sender,
            MyError::IncorrectSender
        );
        require!(
            stream_account.is_cancelled == false,
            MyError::StreamAlreadyCancelled
        );

        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;
        let start = stream_account.start_time;
        let stop = stream_account.stop_time;
        let interval = stream_account.interval;

        if timestamp < start || stream_account.is_paused == true {
            let total_balance = stream_account.remaining_balance as u64;

            **stream_account.to_account_info().try_borrow_mut_lamports()? -= total_balance;
            **ctx.accounts.sender.try_borrow_mut_lamports()? += total_balance;
            stream_account.remaining_balance = 0;
        } else {
            let mut ready_for_withdrawal: u128;

            if timestamp >= stop {
                ready_for_withdrawal = stream_account.remaining_balance;
            } else {
                let delta = timestamp - start;

                let no_of_intervals = delta / interval;

                ready_for_withdrawal = no_of_intervals * stream_account.rate_of_stream;

                if stream_account.deposit > stream_account.remaining_balance {
                    let amt_withdrawn = stream_account.deposit - stream_account.remaining_balance;
                    ready_for_withdrawal -= amt_withdrawn;
                }
            }

            let total_balance = stream_account.remaining_balance as u64;
            let recipient_balance = ready_for_withdrawal as u64;
            let sender_balance = total_balance - recipient_balance;

            **stream_account.to_account_info().try_borrow_mut_lamports()? -= total_balance;
            **ctx.accounts.recipient.try_borrow_mut_lamports()? += recipient_balance;
            **ctx.accounts.sender.try_borrow_mut_lamports()? += sender_balance;

            stream_account.remaining_balance = 0;
        }
        stream_account.is_cancelled = true;
        Ok(())
    }

    pub fn cancel_stream_token(ctx: Context<CancelStreamToken>, stream_id: String) -> Result<()> {
        let stream_account = &mut ctx.accounts.stream;

        require!(
            (stream_account.sender == ctx.accounts.authority.key()
                && stream_account.cancel_by != StateChangeAuth::OnlyReceiver)
                || (stream_account.recipient == ctx.accounts.authority.key()
                    && stream_account.cancel_by != StateChangeAuth::OnlySender),
            MyError::NotAuthorized
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(
            ctx.accounts.token_address.key() == stream_account.token_address,
            MyError::IncorrectTokenAddress
        );
        require!(
            stream_account.is_cancelled == false,
            MyError::StreamAlreadyCancelled
        );
        let recipient_tokens = get_associated_token_address(&ctx.accounts.recipient.key(), &ctx.accounts.token_address.key());
        require!(ctx.accounts.recipient_tokens.key() == recipient_tokens, MyError::AssociatedTokenAccountIncorrect);

        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;
        let start = stream_account.start_time;
        let stop = stream_account.stop_time;
        let interval = stream_account.interval;

        let sender = stream_account.sender;

        let seeds = &[
            stream_id.as_bytes(),
            sender.as_ref(),
            &[stream_account.bump],
        ];

        if timestamp < start || stream_account.is_paused == true {
            let total_balance = stream_account.remaining_balance as u64;

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stream_tokens.clone().to_account_info(),
                        to: ctx.accounts.sender_tokens.to_account_info(),
                        authority: stream_account.to_account_info(),
                    },
                    &[&seeds[..]],
                ),
                total_balance,
            )?;
            stream_account.remaining_balance = 0;
        } else {
            let mut ready_for_withdrawal: u128;

            if timestamp >= stop {
                ready_for_withdrawal = stream_account.remaining_balance;
            } else {
                let delta = timestamp - start;

                let no_of_intervals = delta / interval;

                ready_for_withdrawal = no_of_intervals * stream_account.rate_of_stream;

                if stream_account.deposit > stream_account.remaining_balance {
                    let amt_withdrawn = stream_account.deposit - stream_account.remaining_balance;
                    ready_for_withdrawal -= amt_withdrawn;
                }
            }

            let total_balance = stream_account.remaining_balance as u64;
            let recipient_balance = ready_for_withdrawal as u64;
            let sender_balance = total_balance - recipient_balance;

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stream_tokens.clone().to_account_info(),
                        to: ctx.accounts.recipient_tokens.to_account_info(),
                        authority: stream_account.to_account_info(),
                    },
                    &[&seeds[..]],
                ),
                recipient_balance,
            )?;

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stream_tokens.clone().to_account_info(),
                        to: ctx.accounts.sender_tokens.to_account_info(),
                        authority: stream_account.to_account_info(),
                    },
                    &[&seeds[..]],
                ),
                sender_balance,
            )?;
            stream_account.remaining_balance = 0;
        }
        stream_account.is_cancelled = true;
        Ok(())
    }

    pub fn pause_stream(ctx: Context<PauseStream>, stream_id: String) -> Result<()> {
        let stream_account = &mut ctx.accounts.stream;

        require!(
            (stream_account.sender == ctx.accounts.authority.key()
                && stream_account.pause_by != StateChangeAuth::OnlyReceiver)
                || (stream_account.recipient == ctx.accounts.authority.key()
                    && stream_account.pause_by != StateChangeAuth::OnlySender),
            MyError::NotAuthorized
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(
            stream_account.is_paused == false,
            MyError::StreamAlreadyPaused
        );
        require!(
            stream_account.is_cancelled == false,
            MyError::StreamAlreadyCancelled
        );

        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;
        let start = stream_account.start_time;
        let stop = stream_account.stop_time;
        let interval = stream_account.interval;

        require!(timestamp < stop, MyError::StreamAlreadyEnded);
        require!(timestamp >= start, MyError::StreamNotStarted);

        let delta: u128 = timestamp - start;
        let time_left: u128 = stop - timestamp;

        let no_of_intervals = delta / interval;

        let mut ready_for_withdrawal = no_of_intervals * stream_account.rate_of_stream;

        if stream_account.deposit > stream_account.remaining_balance {
            let amt_withdrawn = stream_account.deposit - stream_account.remaining_balance;
            ready_for_withdrawal -= amt_withdrawn;
        }

        let recipient_balance = ready_for_withdrawal as u64;
        let paused_amount = stream_account.remaining_balance - ready_for_withdrawal;

        if ready_for_withdrawal > 0 {
            **stream_account.to_account_info().try_borrow_mut_lamports()? -= recipient_balance;
            **ctx.accounts.recipient.try_borrow_mut_lamports()? += recipient_balance;
        }

        stream_account.is_paused = true;
        stream_account.time_left = time_left;
        stream_account.remaining_balance = paused_amount;
        stream_account.deposit = paused_amount;

        Ok(())
    }

    pub fn pause_stream_token(ctx: Context<PauseStreamToken>, stream_id: String) -> Result<()> {
        let stream_account = &mut ctx.accounts.stream;

        require!(
            (stream_account.sender == ctx.accounts.authority.key()
                && stream_account.pause_by != StateChangeAuth::OnlyReceiver)
                || (stream_account.recipient == ctx.accounts.authority.key()
                    && stream_account.pause_by != StateChangeAuth::OnlySender),
            MyError::NotAuthorized
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(
            ctx.accounts.recipient.key() == stream_account.recipient,
            MyError::IncorrectRecipient
        );
        require!(
            ctx.accounts.token_address.key() == stream_account.token_address,
            MyError::IncorrectTokenAddress
        );
        require!(
            ctx.accounts.sender.key() == stream_account.sender,
            MyError::IncorrectSender
        );
        require!(
            stream_account.is_paused == false,
            MyError::StreamAlreadyPaused
        );
        require!(
            stream_account.is_cancelled == false,
            MyError::StreamAlreadyEnded
        );
        let recipient_tokens = get_associated_token_address(&ctx.accounts.recipient.key(), &ctx.accounts.token_address.key());
        require!(ctx.accounts.recipient_tokens.key() == recipient_tokens, MyError::AssociatedTokenAccountIncorrect);

        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;
        let start = stream_account.start_time;
        let stop = stream_account.stop_time;
        let interval = stream_account.interval;

        require!(timestamp < stop, MyError::StreamAlreadyEnded);
        require!(timestamp >= start, MyError::StreamNotStarted);

        let delta: u128 = timestamp - start;
        let time_left: u128 = stop - timestamp;

        let no_of_intervals = delta / interval;

        let mut ready_for_withdrawal = no_of_intervals * stream_account.rate_of_stream;

        if stream_account.deposit > stream_account.remaining_balance {
            let amt_withdrawn = stream_account.deposit - stream_account.remaining_balance;
            ready_for_withdrawal -= amt_withdrawn;
        }

        let recipient_balance = ready_for_withdrawal as u64;
        let paused_amount = stream_account.remaining_balance - ready_for_withdrawal;

        let sender = ctx.accounts.sender.key();
        let bump = stream_account.bump;

        let seeds = &[stream_id.as_bytes(), sender.as_ref(), &[bump]];

        if ready_for_withdrawal > 0 {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stream_tokens.clone().to_account_info(),
                        to: ctx.accounts.recipient_tokens.to_account_info(),
                        authority: stream_account.to_account_info(),
                    },
                    &[&seeds[..]],
                ),
                recipient_balance,
            )?;
        }

        stream_account.is_paused = true;
        stream_account.time_left = time_left;
        stream_account.remaining_balance = paused_amount;
        stream_account.deposit = paused_amount;

        Ok(())
    }

    pub fn resume_stream(ctx: Context<ResumeStream>, stream_id: String) -> Result<()> {
        // Get Account
        let stream_account = &mut ctx.accounts.stream;

        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;

        require!(
            (stream_account.sender == ctx.accounts.authority.key()
                && stream_account.resume_by != StateChangeAuth::OnlyReceiver)
                || (stream_account.recipient == ctx.accounts.authority.key()
                    && stream_account.resume_by != StateChangeAuth::OnlySender),
            MyError::NotAuthorized
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(
            timestamp >= stream_account.start_time,
            MyError::StreamNotStarted
        );
        require!(stream_account.is_paused == true, MyError::StreamNotPaused);

        let stop = timestamp + stream_account.time_left;

        stream_account.start_time = timestamp;
        stream_account.stop_time = stop;
        stream_account.is_paused = false;

        Ok(())
    }

    pub fn reload_stream(
        ctx: Context<ReloadStream>,
        stream_id: String,
        amount: u128,
    ) -> Result<()> {
        // Get Account
        let stream_account = &mut ctx.accounts.stream;
        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;

        require!(
            stream_account.is_infinite == true,
            MyError::NotInfiniteStream
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(amount > 0, MyError::DepositIsZero);
        require!(
            timestamp < stream_account.stop_time,
            MyError::StreamAlreadyEnded
        );

        let rate = stream_account.rate_of_stream;
        let interval = stream_account.interval;

        /* Without this, the duration would be zero. */
        require!(amount >= rate, MyError::DepositSmallerThanTime);

        let duration = (amount as f32 / rate as f32) * interval as f32;
        let new_stop = stream_account.stop_time + duration as u128;

        stream_account.stop_time = new_stop;
        stream_account.remaining_balance += amount;
        stream_account.deposit += amount;

        let sender = ctx.accounts.sender.key();

        let seeds = &[
            stream_id.as_bytes(),
            sender.as_ref(),
            &[stream_account.bump],
        ];

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.sender.key(),
            &stream_account.key(),
            amount as u64,
        );

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.sender.to_account_info(),
                stream_account.to_account_info(),
            ],
            &[&seeds[..]],
        )?;

        Ok(())
    }

    pub fn reload_stream_token(
        ctx: Context<ReloadStreamToken>,
        stream_id: String,
        amount: u128,
    ) -> Result<()> {
        // Get Account
        let stream_account = &mut ctx.accounts.stream;
        let clock: Clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp as u128;

        require!(
            stream_account.is_infinite == true,
            MyError::NotInfiniteStream
        );
        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(amount > 0, MyError::DepositIsZero);
        require!(
            timestamp < stream_account.stop_time,
            MyError::StreamAlreadyEnded
        );

        let rate = stream_account.rate_of_stream;
        let interval = stream_account.interval;

        /* Without this, the duration would be zero. */
        require!(amount >= rate, MyError::DepositSmallerThanTime);

        let duration = (amount as f32 / rate as f32) * interval as f32;
        let new_stop = stream_account.stop_time + duration as u128;

        stream_account.stop_time = new_stop;
        stream_account.remaining_balance += amount;
        stream_account.deposit += amount;

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.sender_tokens.clone().to_account_info(),
                    to: ctx.accounts.stream_tokens.to_account_info(),
                    authority: ctx.accounts.sender.clone().to_account_info(),
                },
            ),
            amount as u64,
        )?;

        Ok(())
    }

    pub fn delete_stream(ctx: Context<DeleteStream>, stream_id: String) -> Result<()> {
        // Get Account
        let stream_account = &mut ctx.accounts.stream;
        let stream_list_sender = &mut ctx.accounts.stream_list_sender;
        let stream_list_recipient = &mut ctx.accounts.stream_list_recipient;

        require!(
            stream_account.stream_id == stream_id,
            MyError::IncorrectStreamId
        );
        require!(
            stream_account.remaining_balance == 0,
            MyError::StreamNotEmpty
        );
        require!(
            stream_account.sender == ctx.accounts.sender.key(),
            MyError::NotAuthorized
        );

        stream_account.close(ctx.accounts.sender.to_account_info())?;

        let streamlistsender = StreamItem {
            stream_list: ctx.accounts.stream.key(),
            is_sender: true,
        };
        let streamlistrecipient = StreamItem {
            stream_list: ctx.accounts.stream.key(),
            is_sender: false,
        };

        let x = stream_list_sender
            .items
            .iter()
            .position(|r| r == &streamlistsender)
            .unwrap_or(99999999);
        if x != 99999999 {
            stream_list_sender.items.swap_remove(x);
        }

        let y = stream_list_recipient
            .items
            .iter()
            .position(|r| r == &streamlistrecipient)
            .unwrap_or(99999999);
        if y != 99999999 {
            stream_list_recipient.items.swap_remove(y);
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(stream_id: String)]
pub struct CreateStream<'info> {
    // stream_account Account PDA
    #[account(
        init,
        seeds = [stream_id.as_bytes(), sender.key().as_ref()],
        bump,
        payer = sender,
        space = 16 + StreamAccount::MAX_SIZE
    )]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(
        init_if_needed,
        seeds = [b"streamlist", sender.key().as_ref()],
        bump,
        payer = sender,
        space = 8 + StreamList::MAX_SIZE,
    )]
    pub stream_list_sender: Account<'info, StreamList>,
    #[account(
        init_if_needed,
        seeds = [b"streamlist", recipient.key().as_ref()],
        bump,
        payer = sender,
        space = 8 + StreamList::MAX_SIZE,
    )]
    pub stream_list_recipient: Account<'info, StreamList>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(stream_id: String)]
pub struct CreateStreamToken<'info> {
    // stream_account Account PDA
    #[account(
        init,
        seeds = [stream_id.as_bytes(), sender.key().as_ref()],
        bump,
        payer = sender,
        space = 16 + StreamAccount::MAX_SIZE
    )]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    pub token_address: Account<'info, Mint>,
    #[account(mut)]
    pub sender_tokens: Account<'info, TokenAccount>,
    #[account(init_if_needed,
        payer = sender, 
        associated_token::mint = token_address, 
        associated_token::authority = stream)]
    pub stream_tokens: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        seeds = [b"streamlist", sender.key().as_ref()],
        bump,
        payer = sender,
        space = 8 + StreamList::MAX_SIZE,
    )]
    pub stream_list_sender: Box<Account<'info, StreamList>>,
    #[account(
        init_if_needed,
        seeds = [b"streamlist", recipient.key().as_ref()],
        bump,
        payer = sender,
        space = 8 + StreamList::MAX_SIZE,
    )]
    pub stream_list_recipient: Box<Account<'info, StreamList>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct WithdrawFromStream<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFromStreamToken<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(
        mut,
        token::mint = token_address,
        token::authority = stream
    )]
    pub stream_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub sender: AccountInfo<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(init_if_needed,
        payer = authority, 
        associated_token::mint = token_address, 
        associated_token::authority = recipient)]
    pub recipient_tokens: Account<'info, TokenAccount>,
    pub token_address: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CancelStream<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub sender: AccountInfo<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelStreamToken<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(
        mut,
        token::mint = token_address,
        token::authority = stream
    )]
    pub stream_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(init_if_needed,
        payer = authority, 
        associated_token::mint = token_address, 
        associated_token::authority = recipient)]
    pub recipient_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sender_tokens: Box<Account<'info, TokenAccount>>,
    pub token_address: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct PauseStream<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PauseStreamToken<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(
        mut,
        token::mint = token_address,
        token::authority = stream
    )]
    pub stream_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub sender: AccountInfo<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    #[account(init_if_needed,
        payer = authority, 
        associated_token::mint = token_address, 
        associated_token::authority = recipient)]
    pub recipient_tokens: Account<'info, TokenAccount>,
    pub token_address: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ResumeStream<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReloadStream<'info> {
    // stream_account Account PDA
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReloadStreamToken<'info> {
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(
        mut,
        token::mint = token_address,
        token::authority = stream
    )]
    pub stream_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sender: Signer<'info>,
    #[account(mut)]
    pub sender_tokens: Account<'info, TokenAccount>,
    pub token_address: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DeleteStream<'info> {
    // stream_account Account PDA
    #[account(mut)]
    pub stream: Account<'info, StreamAccount>,
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK: safe
    #[account(mut)]
    pub stream_list_sender: Account<'info, StreamList>,
    /// CHECK: safe
    #[account(mut)]
    pub stream_list_recipient: Account<'info, StreamList>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct StreamAccount {
    // Stream Identifier
    pub stream_id: String,
    // Stream Title
    pub stream_title: String,
    // Recipient address
    pub recipient: Pubkey,
    // Sender address
    pub sender: Pubkey,
    // Token
    pub token_address: Pubkey,
    // Stream start time
    pub start_time: u128,
    // Stream end time
    pub stop_time: u128,
    // Balance Remaining
    pub remaining_balance: u128,
    // Total Deposit
    pub deposit: u128,
    // Interval of Stream
    pub interval: u128,
    // Rate per second
    pub rate_of_stream: u128,
    // Bump
    pub bump: u8,
    // Who can Cancel the Stream
    pub cancel_by: StateChangeAuth,
    // Who can Pause the Stream
    pub pause_by: StateChangeAuth,
    // Who can Resume the Stream
    pub resume_by: StateChangeAuth,
    // Who can Withdraw from the Stream,
    pub withdraw_by: StateChangeAuth,
    // Who can Edit the Stream,
    pub edit_by: StateChangeAuth,
    // Status of Stream
    pub is_paused: bool,
    // Can this stream be deleted
    pub is_cancelled: bool,
    // Infinite Stream
    pub is_infinite: bool,
    // Pause Timestamp
    pub time_left: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum StateChangeAuth {
    OnlySender,
    OnlyReceiver,
    Both,
}

impl StreamAccount {
    pub const MAX_SIZE: usize = (4 + (4 * 4))
        + (4 + (50 * 4))
        + 32
        + 32
        + 32
        + 16
        + 16
        + 16
        + 16
        + 16
        + 16
        + 1
        + (1 + 1)
        + (1 + 1)
        + (1 + 1)
        + (1 + 1)
        + (1 + 1)
        + 1
        + 1
        + 1
        + 16;
}

#[account]
pub struct StreamList {
    pub stream_id: u16,
    pub items: Vec<StreamItem>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Default, PartialEq)]
pub struct StreamItem {
    // StreamList address
    pub stream_list: Pubkey,
    // Status of Stream
    pub is_sender: bool,
}

impl StreamList {
    pub const MAX_SIZE: usize = 2 + 4 + 100 * (32 + 1);
}

#[error_code]
pub enum MyError {
    #[msg("Recipient cannot be same as Sender.")]
    SenderIsRecipient,
    #[msg("Deposit Amount is Zero")]
    DepositIsZero,
    #[msg("Interval of Streaming is Zero")]
    IntervalIsZero,
    #[msg("Amount is Zero")]
    AmountIsZero,
    #[msg("Start Time is before Block Timestamp.")]
    PastStartTime,
    #[msg("Start Time is after the Stop Time.")]
    FutureStartTime,
    #[msg("Deposit is smaller than the Time Delta")]
    DepositSmallerThanTime,
    #[msg("Deposit is not a Multiple of the Time Delta")]
    DepositNotMultipleOfTime,
    #[msg("The Stream ID is Incorrect")]
    IncorrectStreamId,
    #[msg("Stream Title cannot be longer than 50 characters.")]
    TitleTooLong,
    #[msg("Nothing To Withdraw as of now.")]
    NothingToWithdraw,
    #[msg("Incorrect Recipient Address")]
    IncorrectRecipient,
    #[msg("Incorrect Sender Address")]
    IncorrectSender,
    #[msg("Incorrect Token Address")]
    IncorrectTokenAddress,
    #[msg("Stream has not started yet.")]
    StreamNotStarted,
    #[msg("Stream is Paused. Resume the Stream to Withdraw.")]
    StreamIsPaused,
    #[msg("Stream is Already Paused.")]
    StreamAlreadyPaused,
    #[msg("Stream is not Paused.")]
    StreamNotPaused,
    #[msg("Stream has already Ended.")]
    StreamAlreadyEnded,
    #[msg("Stream is already Cancelled.")]
    StreamAlreadyCancelled,
    #[msg("Stream Not Empty. Withdraw Tokens completely and then try again.")]
    StreamNotEmpty,
    #[msg("This is not an infinite stream.")]
    NotInfiniteStream,
    #[msg("You are not Authorized to perform the desired operation. !!")]
    NotAuthorized,
    #[msg("Invalid Value for Cancel By Flag")]
    InvalidCancelBy,
    #[msg("Invalid Value for Pause By Flag")]
    InvalidPauseBy,
    #[msg("Invalid Value for Withdraw By Flag")]
    InvalidWithdrawBy,
    #[msg("Invalid Value for Resume By Flag")]
    InvalidResumeBy,
    #[msg("Invalid Value for Edit By Flag")]
    InvalidEditBy,
    #[msg("The Duration is incorrect. Please check the values of Amount, Rate, Interval and Duration.")]
    IncorrectDuration,
    #[msg("The Associated Token Account of Recipient is Incorrect.")]
    AssociatedTokenAccountIncorrect,
}
