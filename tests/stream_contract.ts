import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { StreamContract } from "../target/types/stream_contract";

describe("stream-contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.StreamContract as Program<StreamContract>;

  it("Is initialized!", async () => {
    // Add your test here.
  });
});
