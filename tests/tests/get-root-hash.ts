import { ContractPromise } from "@polkadot/api-contract";
import { hexStringToBytes } from "./utils";

/** Read last root hash from contract */
export async function getRootHash(
  contract: ContractPromise,
  signerAddress: string
): Promise<Uint8Array> {
  const gasLimit = -1;
  const storageDepositLimit = null;

  const { result, output } = await contract.query.getRootHash(signerAddress, {
    gasLimit,
    storageDepositLimit,
  });

  expect(result.isOk).toBe(true);

  if (output === null) {
    throw Error("Output is null");
  }
 
  // Get hex string from output and convert it to the bytes array
  return hexStringToBytes(output.toHex());
}
