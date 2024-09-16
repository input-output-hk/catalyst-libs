# Test data for CIP-0509 RBAC validation

## Role Based Access Control (RBAC) validation

The current validation includes

1. Hashing the transaction input `0: transaction inputs` within the transaction should match the `1: txn-inputs-hash`.
2. `?7: auxiliary_data_hash` should match the hash of the auxiliary data.
  This also log the pre-hashed of the auxiliary data (auxiliary data with `99: validation signature` set to 0x00).
3. Role 0 validation
    1. Stake public key extracted from URI in X509 or C509 subject alternative name should match some of the
        witness set within the transaction.
    2. Reference `?3: payment-key`
        1. Negative index reference - reference to transaction output in transaction: should match some of the
            key within witness set.
        2. Positive index reference - reference to transaction input in transaction: check only the index
            exist within the transaction input.

## Test Data

* `conway_1.block`: Block number: `2694583`, Absolute slot number: `70795216`
* `conway_2.block`: Block number: `2625015`, Absolute slot number: `68906742`
* `conway_3.block`: Block number: `2694587`, Absolute slot number: `70795304`

## References

* [Conway CDDL](https://github.com/IntersectMBO/cardano-ledger/blob/ab8d57cf43be912a336e872b68d1a2526c93dc6a/eras/conway/impl/cddl-files/conway.cddl)

* [CIP-0509 RBAC Registration](https://github.com/input-output-hk/catalyst-CIPs/tree/x509-rbac-signing-with-cip30/CIP-XXXX)
* [CIP-0509 Metadata Envelope](https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX)
* [CIP-0509 Role Registration](https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX/)

**Note: CIP-0509 is still in process and may be subject to change.**
