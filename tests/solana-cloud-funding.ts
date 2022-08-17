import * as anchor from "@project-serum/anchor";
import { Program, utils, web3, BN } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { SolanaCloudFunding } from "../target/types/solana_cloud_funding";
import * as assert from "assert";

describe("solana-cloud-funding", () => {
  // Use a local provider.
  const provider = anchor.AnchorProvider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaCloudFunding as Program<SolanaCloudFunding>;

  it("Cloud Funding Successful", async () => {
    // findProgramAddressでPDAを取得
    // cloud fundingサービスでは、取引情報はPDAに格納する
    // https://solana-labs.github.io/solana-web3.js/classes/PublicKey.html#findProgramAddress
    const [campaign] = await PublicKey.findProgramAddress(
      [
        utils.bytes.utf8.encode("CAMPAIGN_DEMO"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.rpc.create("campaign name", "campaign description", {
      accounts: {
        campaign,
        user: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      },
    });

    let account = await program.account.campaign.fetch(campaign);
    assert.ok(account.name === "campaign name");
    assert.ok(account.description === "campaign description");
    assert.ok(account.amountDonated.eq(new anchor.BN(0)));

    // 0.2 * web3.LAMPORTS_PER_SOLは預けるお金
    await program.rpc.donate(new BN(0.2 * web3.LAMPORTS_PER_SOL), {
      accounts: {
        campaign,
        user: provider.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      },
    });

    account = await program.account.campaign.fetch(campaign);
    assert.ok(account.amountDonated.eq(new BN(0.2 * web3.LAMPORTS_PER_SOL)));

    // 0.2 * web3.LAMPORTS_PER_SOLは引き出すお金
    const result = await program.rpc.withdraw(new BN(0.2 * web3.LAMPORTS_PER_SOL), {
      accounts: {
        campaign,
        user: provider.wallet.publicKey,
      },
    });

    account = await program.account.campaign.fetch(campaign);
    assert.ok(account.amountDonated.eq(new anchor.BN(0)));
  });
});
