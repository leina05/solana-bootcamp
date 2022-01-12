import argparse
from typing import NamedTuple
import struct
import base64
from nacl.utils import random
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.rpc.api import Client
from solana.rpc.types import TxOpts
from solana.rpc.commitment import Confirmed
from solana.system_program import CreateAccountParams, create_account, SYS_PROGRAM_ID
from solana.transaction import AccountMeta, TransactionInstruction, Transaction
from random import randbytes


pack_str = lambda s: struct.pack("<I" + (len(s) * "B"), len(s), *s.encode("ascii"))
pack_u64 = lambda u: struct.pack("<Q", u)

# Echo instruction parameters
class EchoParams(NamedTuple):
    program_id: PublicKey
    echo_buffer: PublicKey
    data: str


# Creates the echo instruction
def echo(params: EchoParams) -> TransactionInstruction:
    data = b"".join([struct.pack("<B", 0), pack_str(params.data)])

    return TransactionInstruction(
        keys=[
            AccountMeta(pubkey=params.echo_buffer, is_signer=False, is_writable=True)
        ],
        program_id=params.program_id,
        data=data,
    )


# Creates the InitializeAuthorizedEcho instruction
def initialize_authorized_buffer(
    authorized_buffer_pk, authority_pk, program_id, buffer_seed, buffer_size
):
    # pack data
    data = b"".join(
        [struct.pack("<B", 1), pack_u64(buffer_seed), pack_u64(buffer_size)]
    )

    return TransactionInstruction(
        keys=[
            AccountMeta(pubkey=authorized_buffer_pk, is_signer=False, is_writable=True),
            AccountMeta(pubkey=authority_pk, is_signer=True, is_writable=False),
            AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
        ],
        program_id=program_id,
        data=data,
    )


def test_echo(fee_payer, program_id):
    buffer = Keypair()
    create_account_ix = create_account(
        CreateAccountParams(
            from_pubkey=fee_payer.public_key,
            new_account_pubkey=buffer.public_key,
            lamports=client.get_minimum_balance_for_rent_exemption(len(args.echo))[
                "result"
            ],
            space=len(args.echo),
            program_id=program_id,
        )
    )
    # Test echo instruction (0)
    echo_ix = echo(
        EchoParams(
            program_id=program_id,
            echo_buffer=buffer.public_key,
            data=args.echo,
        )
    )

    tx = Transaction().add(create_account_ix).add(echo_ix)
    signers = [fee_payer, buffer]
    result = client.send_transaction(
        tx,
        *signers,
        opts=TxOpts(
            skip_preflight=True,
        ),
    )
    tx_hash = result["result"]
    client.confirm_transaction(tx_hash, commitment="confirmed")

    print(f"https://explorer.solana.com/tx/{tx_hash}?cluster=devnet")

    acct_info = client.get_account_info(buffer.public_key, commitment=Confirmed)
    if acct_info["result"]["value"] is None:
        raise RuntimeError(f"Failed to get account. address={buffer.public_key}")
    data = base64.b64decode(acct_info["result"]["value"]["data"][0]).decode("ascii")
    print("Echo Buffer Text:", data)


def test_authorized_echo(fee_payer, program_id):
    authority_pk = fee_payer.public_key
    buffer_seed = randbytes(8)
    buffer_size = 32
    seeds = [b"authority", bytes(authority_pk), randbytes(8)]
    authorized_buffer_pk, bump_seed = PublicKey.find_program_address(seeds, program_id)
    init_auth_echo_ix = initialize_authorized_buffer(
        authorized_buffer_pk, authority_pk, program_id, buffer_seed, buffer_size
    )

    tx = Transaction().add(init_auth_echo_ix)
    signers = [fee_payer]
    result = client.send_transaction(
        tx,
        *signers,
        opts=TxOpts(
            skip_preflight=True,
        ),
    )
    tx_hash = result["result"]
    client.confirm_transaction(tx_hash, commitment="confirmed")

    print(f"https://explorer.solana.com/tx/{tx_hash}?cluster=devnet")

    acct_info = client.get_account_info(authorized_buffer_pk, commitment=Confirmed)
    if acct_info["result"]["value"] is None:
        raise RuntimeError(f"Failed to get account. address={authorized_buffer_pk}")
    data = base64.b64decode(acct_info["result"]["value"]["data"][0]).decode("ascii")
    print("Echo Buffer Text:", data)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "program_id",
        help="Devnet program ID (base58 encoded string) of the deployed Echo Program",
    )
    parser.add_argument("echo", help="The string to copy on-chain")
    args = parser.parse_args()

    program_id = PublicKey(args.program_id)
    fee_payer = Keypair()
    client = Client("https://api.devnet.solana.com")
    print("Requesting Airdrop of 1 SOL...")
    client.request_airdrop(fee_payer.public_key, int(1e9))
    print("Airdrop received")

    test_echo(fee_payer, program_id)
    test_authorized_echo(fee_payer, program_id)
