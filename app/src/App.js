import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-wallets";
import {
  useAnchorWallet,
  WalletProvider,
  ConnectionProvider,
} from "@solana/wallet-adapter-react";
import {
  WalletModalProvider,
  WalletMultiButton,
} from "@solana/wallet-adapter-react-ui";

import "./App.css";
import { useMemo } from "react";
import {
  clusterApiUrl,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  Program,
  AnchorProvider,
  web3,
  utils,
  BN,
} from "@project-serum/anchor";
import idl from "./idl.json";
import * as buffer from "buffer";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Metaplex, AccountNotFoundError } from "@metaplex-foundation/js";

window.Buffer = buffer.Buffer;

const utf8 = utils.bytes.utf8;
const recipient = new PublicKey("7QmBg2FW8uXy7GrnHMcHeoFJ9qfT7zRRnTkRG1EDSzeM");
const tokenAddress = new PublicKey(
  "Gssm3vfi8s65R31SBdmQRq6cKeYojGgup7whkw4VCiQj"
);
const streamPDA = new PublicKey("FzmNArJLjHofiuG9KowbSk51jVzzPUDs5dFmNJqkbvWF");
const streamPDAtoken = new PublicKey(
  "E1wpegQ9P5rbCjbNDfHeapfcFeCaEiLfybzkuyfCgknN"
);
const streamPDAtoken2 = new PublicKey(
  "2ezLde3TbC7DFcPBpkh2TWoGEsW8GqezEnmZreaVszaq"
);
const BLANK = "                                ";

require("@solana/wallet-adapter-react-ui/styles.css");

const opts = {
  preflightCommitment: "processed",
};
const programID = new PublicKey(idl.metadata.address);

const App = () => {
  return (
    <Context>
      <Content />
    </Context>
  );
};
export default App;

const Context = ({ children }) => {
  // The network can be set to 'devnet', 'testnet', or 'mainnet-beta'.
  const network = WalletAdapterNetwork.Devnet;

  // You can also provide a custom RPC endpoint.
  const endpoint = useMemo(() => clusterApiUrl(network), [network]);

  const wallets = useMemo(() => [new PhantomWalletAdapter()], []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
};

const Content = () => {
  const wallet = useAnchorWallet();

  async function getProvider() {
    /* create the provider and return it to the caller */
    /* network set to local network for now */
    const network = "https://api.devnet.solana.com";

    const connection = new Connection(network, opts.preflightCommitment);

    const provider = new AnchorProvider(
      connection,
      wallet,
      opts.preflightCommitment
    );
    return provider;
  }

  async function createStream(
    recipient,
    tokenA,
    title,
    deposit,
    startTime,
    interval,
    ratePerInterval,
    duration,
    isInfinite,
    cancel,
    pause,
    resume,
    withdraw,
    edit,
    startNow
  ) {
    const provider = await getProvider();
    const network = "https://api.devnet.solana.com";
    const connection = new Connection(network, opts.preflightCommitment);
    const program = new Program(idl, programID, provider);

    try {
      const startTimestamp = new Date(startTime).valueOf() / 1000;
      const recipientKey = new PublicKey(recipient);
      var streamId = 0;

      const [streamListSender, bumpSender] =
        await web3.PublicKey.findProgramAddress(
          [utf8.encode("streamlist"), provider.wallet.publicKey.toBuffer()],
          program.programId
        );
      const [streamListRecipient, bumpRecipient] =
        await web3.PublicKey.findProgramAddress(
          [utf8.encode("streamlist"), recipientKey.toBuffer()],
          program.programId
        );

      const accountInfo = await connection.getAccountInfo(streamListSender);
      const isExist = accountInfo !== null;

      if (isExist) {
        const streamListSenderAccount = await program.account.streamList.fetch(
          streamListSender
        );
        streamId = streamListSenderAccount.streamId;
      }

      streamId += 1;
      const streamIdString = streamId.toString();

      const [streamPDA, bump] = await web3.PublicKey.findProgramAddress(
        [utf8.encode(streamIdString), provider.wallet.publicKey.toBuffer()],
        program.programId
      );

      console.log("streamPDA: ", streamPDA.toString(), "bump: ", bump);
      console.log(
        "streamListSender: ",
        streamListSender.toString(),
        "bump: ",
        bumpSender
      );
      console.log(
        "streamListRecipient: ",
        streamListRecipient.toString(),
        "bump: ",
        bumpRecipient
      );

      if (!tokenA) {
        const trans = await program.methods
          .createStream(
            streamIdString,
            title,
            new BN(bump),
            new BN(deposit),
            new BN(startTimestamp),
            new BN(interval),
            new BN(ratePerInterval),
            new BN(duration),
            isInfinite,
            new BN(cancel),
            new BN(pause),
            new BN(resume),
            new BN(withdraw),
            new BN(edit),
            startNow
          )
          .accounts({
            stream: streamPDA,
            sender: provider.wallet.publicKey,
            recipient: recipientKey,
            streamListSender: streamListSender,
            streamListRecipient: streamListRecipient,
            systemProgram: web3.SystemProgram.programId,
          })
          .rpc();

        console.log("trans", trans);
      } else {
        const token = new PublicKey(tokenA);
        const senderTokens = await getAssociatedTokenAddress(
          token,
          provider.wallet.publicKey
        );
        const streamTokens = await getAssociatedTokenAddress(
          token,
          streamPDA,
          true
        );

        const values = [
          new BN(deposit),
          new BN(startTimestamp),
          new BN(interval),
          new BN(ratePerInterval),
          new BN(duration),
        ];
        const trans = await program.methods
          .createStreamToken(
            streamIdString,
            title,
            values,
            isInfinite,
            new BN(cancel),
            new BN(pause),
            new BN(resume),
            new BN(withdraw),
            new BN(edit),
            startNow
          )
          .accounts({
            stream: streamPDA,
            sender: provider.wallet.publicKey,
            recipient: recipientKey,
            tokenAddress: token,
            senderTokens: senderTokens,
            streamTokens: streamTokens,
            streamListSender: streamListSender,
            streamListRecipient: streamListRecipient,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        console.log("trans", trans);
      }
      const streamAccount = await program.account.streamAccount.fetch(
        streamPDA
      );
      const streamListSenderAccount = await program.account.streamList.fetch(
        streamListSender
      );
      const streamListRecipientAccount = await program.account.streamList.fetch(
        streamListRecipient
      );

      console.log("Stream Contains: ");
      console.log("Stream ID: ", streamAccount.streamId);
      console.log("Stream Title: ", streamAccount.streamTitle.toString());
      console.log("Sender: ", streamAccount.sender.toString());
      console.log("Recipient: ", streamAccount.recipient.toString());
      if (!tokenA) {
        console.log(
          "tokenAddress: ",
          String.fromCharCode.apply(
            String,
            streamAccount.tokenAddress.toBytes()
          )
        );
      } else {
        console.log("tokenAddress: ", streamAccount.tokenAddress.toString());
      }
      console.log(
        "startTime: ",
        (
          await getDate(Number(streamAccount.startTime.toString()) * 1000)
        ).valueOf()
      );
      console.log(
        "stopTime: ",
        (
          await getDate(Number(streamAccount.stopTime.toString()) * 1000)
        ).valueOf()
      );
      console.log(
        "remainingBalance: ",
        streamAccount.remainingBalance.toString()
      );
      console.log("deposit: ", streamAccount.deposit.toString());
      console.log("Interval: ", streamAccount.interval.toString());
      console.log("ratePerSecond: ", streamAccount.rateOfStream.toString());
      console.log("Bump: ", streamAccount.bump);
      console.log("Is Paused: ", streamAccount.isPaused.toString());
      console.log("Is Infinite: ", streamAccount.isInfinite.toString());
      console.log("Is Cancelled: ", streamAccount.isCancelled.toString());
      console.log("cancel by: ", Object.keys(streamAccount.cancelBy)[0]);
      console.log("pause by: ", Object.keys(streamAccount.pauseBy)[0]);
      console.log("resume by: ", Object.keys(streamAccount.resumeBy)[0]);
      console.log("withdraw by: ", Object.keys(streamAccount.withdrawBy)[0]);
      console.log("edit by: ", Object.keys(streamAccount.editBy)[0]);

      let sender_len = streamListSenderAccount.items.length - 1;
      let rec_len = streamListRecipientAccount.items.length - 1;
      console.log(
        "StreamList Sender contains: ",
        streamListSenderAccount.items[sender_len].streamList.toString(),
        streamListSenderAccount.items[sender_len].isSender.toString()
      );
      console.log(
        "StreamList Recipient contains: ",
        streamListRecipientAccount.items[rec_len].streamList.toString(),
        streamListRecipientAccount.items[rec_len].isSender.toString()
      );
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  async function getDate(timestamp) {
    var dateformat = new Date(timestamp);
    var date =
      dateformat.getDate() +
      "/" +
      (dateformat.getMonth() + 1) +
      "/" +
      dateformat.getFullYear() +
      " " +
      dateformat.getHours() +
      ":" +
      dateformat.getMinutes() +
      ":" +
      dateformat.getSeconds();
    return date;
  }

  async function viewDetails(streamPDA, streamListSender, streamListRecipient) {
    const provider = await getProvider();
    const network = "https://api.devnet.solana.com";
    const connection = new Connection(network, opts.preflightCommitment);

    /* create the program interface combining the idl, program ID, and provider */
    const program = new Program(idl, programID, provider);
    try {
      const a = await connection.getParsedTokenAccountsByOwner(
        provider.wallet.publicKey,
        {
          programId: new PublicKey(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
          ),
        }
      );
      let token_array = [];
      let token_array_final = [];
      const metaplex = Metaplex.make(connection);

      a.value.forEach((account, i) => {
        //Parse the account data
        const parsedAccountInfo = account.account.data;
        const mint = parsedAccountInfo["parsed"]["info"]["mint"];
        const tokenBalance =
          parsedAccountInfo["parsed"]["info"]["tokenAmount"]["uiAmount"];
        //Log results

        token_array.push({
          TokenMint: mint,
          TokenBalance: tokenBalance,
        });
      });
      console.log(token_array);

      for (let i = 0; i < token_array.length; i++) {
        const mintAddress = new PublicKey(token_array[i].TokenMint);
        try {
          const nft = await metaplex.nfts().findByMint({ mintAddress });
          token_array_final.push({
            TokenName: nft.name,
            TokenSymbol: nft.symbol,
            TokenMint: new PublicKey(token_array[i].TokenMint),
            TokenBalance: token_array[i].TokenBalance,
          });
        } catch (error) {
          if (error instanceof AccountNotFoundError) {
            token_array_final.push({
              TokenName: token_array[i].TokenMint.substring(0, 5),
              TokenSymbol: token_array[i].TokenMint.substring(0, 5),
              TokenMint: new PublicKey(token_array[i].TokenMint),
              TokenBalance: token_array[i].TokenBalance,
            });
          }
        }
      }
      console.log(
        "SOL: ",
        await connection.getBalance(provider.wallet.publicKey)
      );
      console.log(token_array_final);

      const streamAct = await program.account.streamAccount.fetch(streamPDA);

      var token = String.fromCharCode.apply(
        String,
        streamAct.tokenAddress.toBytes()
      );
      if (token !== BLANK) {
        token = streamAct.tokenAddress.toString();
      }
      console.log(
        "Details of Account: ",
        streamPDA.toString(),
        " are as follows: "
      );

      console.log("Stream ID: ", streamAct.streamId);
      console.log("Stream Title: ", streamAct.streamTitle.toString());
      console.log("Sender: ", streamAct.sender.toString());
      console.log("Recipient: ", streamAct.recipient.toString());
      console.log("tokenAddress: ", token);
      console.log(
        "startTime: ",
        (await getDate(Number(streamAct.startTime.toString()) * 1000)).valueOf()
      );
      console.log(
        "stopTime: ",
        (await getDate(Number(streamAct.stopTime.toString()) * 1000)).valueOf()
      );
      console.log("remainingBalance: ", streamAct.remainingBalance.toString());
      console.log("deposit: ", streamAct.deposit.toString());
      console.log("interval: ", streamAct.interval.toString());
      console.log("ratePerSecond: ", streamAct.rateOfStream.toString());
      console.log("Bump: ", streamAct.bump);
      console.log("Is Paused: ", streamAct.isPaused.toString());
      console.log("Is Infinite: ", streamAct.isInfinite.toString());
      console.log("Is Cancelled: ", streamAct.isCancelled.toString());
      console.log("cancel by: ", Object.keys(streamAct.cancelBy)[0]);
      console.log("pause by: ", Object.keys(streamAct.pauseBy)[0]);
      console.log("resume by: ", Object.keys(streamAct.resumeBy)[0]);
      console.log("withdraw by: ", Object.keys(streamAct.withdrawBy)[0]);
      console.log("edit by: ", Object.keys(streamAct.editBy)[0]);
      console.log("Current Timestamp: ", new Date().valueOf() / 1000);
      console.log("Time Left: ", streamAct.timeLeft.toString());
      if (new Date().valueOf() / 1000 - streamAct.startTime.toString() >= 0) {
        console.log(
          "Stream Started. Time elapsed: ",
          new Date().valueOf() / 1000 - streamAct.startTime.toString()
        );
      } else {
        console.log("Stream Not Yet Started.");
      }

      const streamListSenderAccount = await program.account.streamList.fetch(
        streamListSender
      );
      const streamListRecipientAccount = await program.account.streamList.fetch(
        streamListRecipient
      );

      console.log(
        "StreamList Sender contains: ",
        streamListSenderAccount.items /*streamListSenderAccount.items[0].isSender.toString()*/
      );
      console.log(
        "StreamList Recipient contains: ",
        streamListRecipientAccount.items /*streamListRecipientAccount.items[0].isSender.toString()*/
      );
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  function getWithdrawAmount(
    start,
    stop,
    interval,
    remBal,
    rate,
    amount
  ) {
    const timestamp = Math.floor(new Date().valueOf() / 1000);
    let readyForWithdrawal;
    if (timestamp >= stop) {
      return remBal;
    } else {
      let delta = timestamp - start;
      if (delta < interval) {
        readyForWithdrawal = 0;
        return readyForWithdrawal;
      } else {
      let no_of_intervals = Math.floor(delta / interval);

      readyForWithdrawal = no_of_intervals * rate;

      if (amount > remBal) {
        let amt_withdrawn = amount - remBal;
        readyForWithdrawal -= amt_withdrawn;
      }
      return readyForWithdrawal;
    }
  }
}

  async function listAllStreams() {
    const provider = await getProvider();
    /* create the program interface combining the idl, program ID, and provider */
    const network = "https://api.devnet.solana.com";

    const connection = new Connection(network, opts.preflightCommitment);
    const program = new Program(idl, programID, provider);
    try {
      const streamListSender = await web3.PublicKey.findProgramAddress(
        [utf8.encode("streamlist"), provider.wallet.publicKey.toBuffer()],
        program.programId
      );

      const accountInfo = await connection.getAccountInfo(streamListSender[0]);
      const isExist = accountInfo !== null;

      if (isExist) {
        const streamListSenderAccount = await program.account.streamList.fetch(
          streamListSender[0]
        );
        var output = [];

        for (let i = 0; i < streamListSenderAccount.items.length; i++) {
          if (streamListSenderAccount.items[i] !== undefined) {
            const streamAct = await program.account.streamAccount.fetch(
              streamListSenderAccount.items[i].streamList
            );
            const start = streamAct.startTime.toString();
            const stop = streamAct.stopTime.toString();
            const interval = streamAct.interval;
            const remBal = streamAct.remainingBalance;
            const rate = streamAct.rateOfStream;
            const amount = streamAct.deposit;
            let readyForWithdrawal = getWithdrawAmount(
              start,
              stop,
              Number(interval.toString()),
              Number(remBal.toString()),
              Number(rate.toString()),
              Number(amount.toString())
            );
            var status = String;
            if (streamAct.isPaused === true) {
              status = "Paused";
            } else if (streamAct.isCancelled === true) {
              status = "Cancelled";
            } else if (
              new Date().valueOf() / 1000 >
              streamAct.stopTime.toString()
            ) {
              status = "Ended";
            } else if (
              new Date().valueOf() / 1000 <
              streamAct.startTime.toString()
            ) {
              status = "Scheduled";
            } else {
              status = "Active";
            }
            var CreatorOrReceiver = String;
            if (
              streamListSenderAccount.items[i].isSender.toString() === "true"
            ) {
              CreatorOrReceiver = "Creator";
            } else {
              CreatorOrReceiver = "Receiver";
            }

            let intervalString = String;
            if (streamAct.interval.toString() === "1"){
              intervalString = "Per Second";
            } else if (streamAct.interval.toString() === "60"){
              intervalString = "Per Minute";
            } else if (streamAct.interval.toString() === "3600"){
              intervalString = "Per Hour";
            } else if (streamAct.interval.toString() === "86400"){
              intervalString = "Per Day";
            } else if (streamAct.interval.toString() === "604800"){
              intervalString = "Per Week";
            } else if (streamAct.interval.toString() === "2592000"){
              intervalString = "Per Month";
            } else if (streamAct.interval.toString() === "31536000"){
              intervalString = "Per Year";
            }
            output.push({
              streamId: streamListSenderAccount.items[i].streamList.toString(),
              title: streamAct.streamTitle.toString(),
              remainingBalance: (Number(streamAct.remainingBalance.toString())/LAMPORTS_PER_SOL).toFixed(2),
              readyForWithdrawal: readyForWithdrawal,
              status: status,
              isContinuous: streamAct.isInfinite,
              Sender: streamAct.sender.toString(),
              Recipient: streamAct.recipient.toString(),
              Interval: intervalString,
              CancelBy: Object.keys(streamAct.cancelBy)[0],
              PauseBy:  Object.keys(streamAct.pauseBy)[0],
              WithdrawBy:  Object.keys(streamAct.withdrawBy)[0],
              StartTime: (
                await getDate(Number(streamAct.startTime.toString()) * 1000)
              ).valueOf(),
              EndTime: (
                await getDate(Number(streamAct.stopTime.toString()) * 1000)
              ).valueOf(),
              CreatorOrReceiver,
            });
          } else {
            break;
          }
        }
        console.log(output);
      }
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  async function withdrawFromStream(stream) {
    const provider = await getProvider();
    /* create the program interface combining the idl, program ID, and provider */
    const program = new Program(idl, programID, provider);
    try {
      const streamPDA = new PublicKey(stream);
      const streamAct = await program.account.streamAccount.fetch(streamPDA);
      const rem_bal = Number(streamAct.remainingBalance.toString());
      if (rem_bal !== 0) {
        var token = String.fromCharCode.apply(
          String,
          streamAct.tokenAddress.toBytes()
        );
        if (token === BLANK) {
          const trans = await program.methods
            .withdrawFromStream(streamAct.streamId.toString())
            .accounts({
              stream: streamPDA,
              authority: provider.wallet.publicKey,
              recipient: streamAct.recipient.toString(),
              systemProgram: web3.SystemProgram.programId,
            })
            .rpc();
          console.log("trans", trans);
        } else {
          token = streamAct.tokenAddress;

          const streamTokens = await getAssociatedTokenAddress(
            token,
            streamPDA,
            true
          );
          const recipientTokens = await getAssociatedTokenAddress(
            token,
            streamAct.recipient
          );

          const trans = await program.methods
            .withdrawFromStreamToken(streamAct.streamId.toString())
            .accounts({
              stream: streamPDA,
              streamTokens: streamTokens,
              authority: provider.wallet.publicKey,
              sender: streamAct.sender.toString(),
              recipient: streamAct.recipient.toString(),
              recipientTokens: recipientTokens,
              tokenAddress: token,
              token_program: TOKEN_PROGRAM_ID,
              systemProgram: web3.SystemProgram.programId,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              rent: SYSVAR_RENT_PUBKEY,
            })
            .rpc();

          console.log("trans", trans);
        }
        const streamAccount = await program.account.streamAccount.fetch(
          streamPDA
        );
        console.log(
          "Remaining Balance in Stream: ",
          streamAccount.remainingBalance.toString()
        );
      } else {
        console.log("Nothing left to withdraw.");
      }
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  async function cancelStream(stream) {
    const provider = await getProvider();
    /* create the program interface combining the idl, program ID, and provider */
    const program = new Program(idl, programID, provider);
    try {
      const streamPDA = new PublicKey(stream);
      const streamAct = await program.account.streamAccount.fetch(streamPDA);
      var token = String.fromCharCode.apply(
        String,
        streamAct.tokenAddress.toBytes()
      );
      if (token === BLANK) {
        const trans = await program.methods
          .cancelStream(streamAct.streamId.toString())
          .accounts({
            stream: streamPDA,
            authority: provider.wallet.publicKey,
            sender: streamAct.sender.toString(),
            recipient: streamAct.recipient.toString(),
            systemProgram: web3.SystemProgram.programId,
          })
          .rpc();
        console.log("trans", trans);
      } else {
        token = streamAct.tokenAddress;

        const streamTokens = await getAssociatedTokenAddress(
          token,
          streamPDA,
          true
        );
        const recipientTokens = await getAssociatedTokenAddress(
          token,
          streamAct.recipient
        );
        const senderTokens = await getAssociatedTokenAddress(
          token,
          streamAct.sender
        );

        const trans = await program.methods
          .cancelStreamToken(streamAct.streamId.toString())
          .accounts({
            stream: streamPDA,
            streamTokens: streamTokens,
            authority: provider.wallet.publicKey,
            recipient: streamAct.recipient.toString(),
            recipientTokens: recipientTokens,
            senderTokens: senderTokens,
            tokenAddress: token,
            token_program: TOKEN_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        console.log("trans", trans);
      }
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  async function pauseStream(stream) {
    const provider = await getProvider();
    /* create the program interface combining the idl, program ID, and provider */
    const program = new Program(idl, programID, provider);
    try {
      const streamPDA = new PublicKey(stream);
      const streamAct = await program.account.streamAccount.fetch(streamPDA);
      var token = String.fromCharCode.apply(
        String,
        streamAct.tokenAddress.toBytes()
      );
      if (token === BLANK) {
        const trans = await program.methods
          .pauseStream(streamAct.streamId.toString())
          .accounts({
            stream: streamPDA,
            authority: provider.wallet.publicKey,
            recipient: streamAct.recipient.toString(),
            systemProgram: web3.SystemProgram.programId,
          })
          .rpc();
        console.log("trans", trans);
      } else {
        token = streamAct.tokenAddress;

        const streamTokens = await getAssociatedTokenAddress(
          token,
          streamPDA,
          true
        );
        const recipientTokens = await getAssociatedTokenAddress(
          token,
          streamAct.recipient
        );

        const trans = await program.methods
          .pauseStreamToken(streamAct.streamId.toString())
          .accounts({
            stream: streamPDA,
            streamTokens: streamTokens,
            authority: provider.wallet.publicKey,
            sender: streamAct.sender.toString(),
            recipient: streamAct.recipient.toString(),
            recipientTokens: recipientTokens,
            tokenAddress: token,
            token_program: TOKEN_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        console.log("trans", trans);
      }
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  async function resumeStream(streamPDA) {
    const provider = await getProvider();
    /* create the program interface combining the idl, program ID, and provider */
    const program = new Program(idl, programID, provider);
    try {
      const streamAct = await program.account.streamAccount.fetch(streamPDA);

      const trans = await program.methods
        .resumeStream(streamAct.streamId.toString())
        .accounts({
          stream: streamPDA,
          authority: provider.wallet.publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc();
      console.log("trans", trans);
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  async function reloadStream(stream, amount) {
    const provider = await getProvider();
    /* create the program interface combining the idl, program ID, and provider */
    const program = new Program(idl, programID, provider);
    try {
      const streamPDA = new PublicKey(stream);
      const streamAct = await program.account.streamAccount.fetch(streamPDA);

      const deposit = amount * LAMPORTS_PER_SOL;
      var token = String.fromCharCode.apply(
        String,
        streamAct.tokenAddress.toBytes()
      );
      if (token === BLANK) {
        const trans = await program.methods
          .reloadStream(streamAct.streamId.toString(), new BN(deposit))
          .accounts({
            stream: streamPDA,
            sender: provider.wallet.publicKey,
            systemProgram: web3.SystemProgram.programId,
          })
          .rpc();
        console.log("trans", trans);
      } else {
        token = streamAct.tokenAddress;

        const streamTokens = await getAssociatedTokenAddress(
          token,
          streamPDA,
          true
        );
        const senderTokens = await getAssociatedTokenAddress(
          token,
          provider.wallet.publicKey
        );

        const trans = await program.methods
          .reloadStreamToken(streamAct.streamId.toString(), new BN(deposit))
          .accounts({
            stream: streamPDA,
            streamTokens: streamTokens,
            sender: streamAct.sender.toString(),
            senderTokens: senderTokens,
            tokenAddress: token,
            token_program: TOKEN_PROGRAM_ID,
          })
          .rpc();

        console.log("trans", trans);
      }
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  async function deleteStream(stream) {
    const provider = await getProvider();
    /* create the program interface combining the idl, program ID, and provider */
    const program = new Program(idl, programID, provider);
    try {
      const streamPDA = new PublicKey(stream);
      const streamAct = await program.account.streamAccount.fetch(streamPDA);

      const streamListSender = await web3.PublicKey.findProgramAddress(
        [utf8.encode("streamlist"), provider.wallet.publicKey.toBuffer()],
        program.programId
      );
      const streamListRecipient = await web3.PublicKey.findProgramAddress(
        [utf8.encode("streamlist"), streamAct.recipient.toBuffer()],
        program.programId
      );

      const trans = await program.methods
        .deleteStream(streamAct.streamId.toString())
        .accounts({
          stream: streamPDA,
          sender: provider.wallet.publicKey,
          streamListSender: streamListSender[0],
          streamListRecipient: streamListRecipient[0],
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc();
      console.log("trans", trans);
    } catch (err) {
      console.log("Transaction error: ", err);
    }
  }

  return (
    <div className="App">
      <button
        onClick={() =>
          createStream(
            recipient,
            tokenAddress,
            "Testing",
            10000000000,
            "2023-01-22 11:57",
            60,
            1000000000,
            600,
            true,
            2,
            1,
            1,
            0,
            2
          )
        }
      >
        Create Stream
      </button>
      <button onClick={() => withdrawFromStream(streamPDAtoken)}>
        Withdraw From Stream
      </button>
      <button onClick={() => cancelStream(streamPDAtoken)}>
        Cancel Stream
      </button>
      <button onClick={() => pauseStream(streamPDAtoken)}>Pause Stream</button>
      <button onClick={() => resumeStream(streamPDAtoken2)}>
        Resume Stream
      </button>
      <button onClick={() => reloadStream(streamPDAtoken, 10)}>
        Reload Stream
      </button>
      <button onClick={() => deleteStream(streamPDAtoken)}>
        Delete Stream
      </button>
      <button
        onClick={() =>
          viewDetails(
            streamPDAtoken,
            "A8qiM741Jru7B2kZjSX5EhiV6my1qLi1GdTfXe1peDfp",
            "ATPGNLDPpqgLbEpoLe73prQe7QFchvnLsAQ4x1NaCX1L"
          )
        }
      >
        View Details
      </button>
      <button onClick={() => listAllStreams()}>List All Streams</button>
      <WalletMultiButton />
    </div>
  );
};
