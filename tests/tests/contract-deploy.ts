import { CodePromise, ContractPromise } from "@polkadot/api-contract";
import { KeyringPair } from "@polkadot/keyring/types";
import { wait } from "./utils";
import metadata from "../../target/ink/slushie/metadata.json";
import { readFileSync } from "fs";
import { ApiPromise } from "@polkadot/api";

/** Deploy contract to local contract node */
export async function deployContract(
  api: ApiPromise,
  signer: KeyringPair,
  depositSize: number
): Promise<ContractPromise> {
  // Read wasm and create code promise
  const wasm = readFileSync("../target/ink/slushie/slushie.wasm");
  const code = new CodePromise(api, metadata, wasm);

  // Create transaction
  const gasLimit = 800000n * 1000000n;
  const storageDepositLimit = null;
  const tx = code.tx.new(
    { gasLimit, storageDepositLimit },
    BigInt(depositSize) * 1000000000000n
  );

  // Send transaction and create contract promise when it will be finished
  let contract;
  const unsub = await tx.signAndSend(signer, (result) => {
    if (result.status.isInBlock || result.status.isFinalized) {
      result.events.forEach(({ event: { data, method } }) => {
        if (method == "NewAccount") {
          const address: string = JSON.parse(data.toString())[0];
          contract = new ContractPromise(api, metadata, address);

          unsub();
        }
      });
    }
  });

  // Wait until contract will be finished
  await wait(400);

  return contract as unknown as ContractPromise;
}
