---
Title: Jörmungandr Voting Transaction (Historical)
Authors:
    - Alex Pozhylenkov <alex.pozhylenkov@iohk.io>
Created: 2024-10-24
---

## Abstract

This document describes a definition of the original Jörmungandr `VoteCast` transaction.
It's documented here for its historical context and to provide more information with regard to the
basis of the construction of the ballot documents in this specification only.

## Motivation

## Specification

An original Jörmungandr blockchain's `VoteCast` transaction structure.

<!-- markdownlint-disable max-one-sentence-per-line code-block-style MD013 -->
??? note "Jormungandr transaction definition: `jorm.abnf`"

    ```abnf
    VOTE-TX           = SIZE-BYTES-32BIT %x00 %x0b VOTE-PAYLOAD

    VOTE-PAYLOAD      = CAST-CERT IOW
    CAST-CERT         = VOTE-PLAN-ID PROPOSAL-INDEX CAST-PAYLOAD

    VOTE-PLAN-ID      = SIZE-BYTES-32BYTE               ; Jörmungandr specific vote plan identifier, Blake2b hash of the vote plan bytes
    PROPOSAL-INDEX    = U8                              ; Jörmungandr specific proposal identifier
    CAST-PAYLOAD      = %x01 CHOICE                     ; Public payload
                       / %x02 ENCRYPTED-VOTE PROOF-VOTE ; Private payload

    CHOICE            = U8
    ENCRYPTED-VOTE    = SIZE-BYTES-8BIT *CIPHERTEXT
    PROOF-VOTE        = SIZE-BYTES-8BIT *ANNOUNCEMENT *CIPHERTEXT *R-RESPONSE SCALAR ; size of the *ANNOUNCEMENT, *CIPHERTEXT, *R-RESPONSE are equal to SIZE-BYTES-8BIT value

    CIPHERTEXT        = E1 E2
    ANNOUNCEMENT      = I A B
    R-RESPONSE        = 3 * SCALAR
    I                 = GROUP-ELEMENT
    A                 = GROUP-ELEMENT
    B                 = GROUP-ELEMENT
    E1                = GROUP-ELEMENT
    E2                = GROUP-ELEMENT

    ; ####################
    ; IOW stand for Inputs-Outputs-Witnesses
    ; ####################

    IOW               = BLOCK-DATE
                        %x01              ; number of inputs and witness
                        %x00              ; number of outputs
                        INPUT             ; one input
                        WITNESS           ; one witness

    INPUT             = %xff
                        VALUE
                        ED25519-PUBLICKEY

    WITNESS           = %x02
                        NONCE
                        ED25519-SIGNATURE

    VALUE                = U64 ; could be anything, not processed anymore, recommended set to zero
    NONCE                = U32 ; could be anything, not processed anymore, recommended set to zero
    BLOCK-DATE           = BLOCK-EPOCH BLOCK-SLOT ; expiration date, could be anything, not processed anymore, recommended set to zeros
    BLOCK-EPOCH          = U32
    BLOCK-SLOT           = U32

    ; ####################
    ; CRYPTO
    ; ####################

    ED25519-PUBLICKEY = SIZE-BYTES-32BYTE
    ED25519-SIGNATURE = SIZE-BYTES-64BYTE

    ; ####################
    ; PRIMITIVES
    ; ####################

    SIZE-BYTES-8BIT    = U8  ; size in elements (8 bits)
    SIZE-BYTES-16BIT   = U16 ; size in bytes (16 bits)
    SIZE-BYTES-32BIT   = U32 ; size in bytes (32 bits)
    U8                 = OCTET   ; unsigned integer 8 bit
    U16                = 2OCTET ; unsigned integer 16 bit (BE)
    U32                = 4OCTET ; unsigned integer 32 bit (BE)
    U64                = 8OCTET ; unsigned integer 64 bit (BE)
    SIZE-BYTES-32BYTE  = 32OCTET ; unsigned integer 256 bit (32 bytes) (BE)
    SIZE-BYTES-64BYTE  = 64OCTET ; unsigned integer 512 bit (64 bytes) (BE)
    SIZE-BYTES-65BYTE  = 65OCTET ; unsigned integer 520 bit (65 bytes) (BE)
    SCALAR             = SIZE-BYTES-32BYTE
    GROUP-ELEMENT      = SIZE-BYTES-32BYTE ; ristretto255 group element
    ```

??? example "Jormungandr transaction representation in hex"

    ```hex
    0000037e000b36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d
    38194c9d4ac7b000203f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5
    a4b286909744746c8b6fb0018773d3b4308344d2e90599cd03749658561787e
    ab714b542a5ccaf078846f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0
    f5a4b286909744746c8b6fc8f58976fc0e951ba284a24f3fc190d914ae53aeb
    cc523e7a4a330c8655b4908f6639bdbc9235103825a9f025eae5cff3bd9c9dc
    c0f5a4b286909744746c8b6fb0018773d3b4308344d2e90599cd03749658561
    787eab714b542a5ccaf078846021c76d0a50054ef7205cb95c1fd3f928f224f
    ab8a8d70feaf4f5db90630c3845a06df2f11c881e396318bd8f9e9f135c2477
    e923c3decfd6be5466d6166fb3c702edd0d1d0a201fb8c51a91d01328da2579
    71ca78cc566d4b518cb2cd261f96644067a7359a745fe239db8e73059883aec
    e4d506be71c1262b137e295ce5f8a0aac22c1d8d343e5c8b5be652573b85cba
    8f4dcb46cfa4aafd8d59974e2eb65f480cf85ab522e23203c4f2faa9f95ebc0
    cd75b04f04fef5d4001d349d1307bb5570af4a91d8af4a489297a3f5255c1e1
    2948787271275c50386ab2ef3980d882228e5f3c82d386e6a4ccf7663df5f6b
    bd9cbbadd6b2fea2668a8bf5603be29546152902a35fc44aae80d9dcd85fad6
    cde5b47a6bdc6257c5937f8de877d5ca0356ee9f12a061e03b99ab9dfea5629
    5485cb5ce38cd37f56c396949f58b0627f455d26e4c5ff0bc61ab0ff05ffa07
    880d0e5c540bc45b527e8e85bb1da469935e0d3ada75d7d41d785d67d1d0732
    d7d6cbb12b23bfc21dfb4bbe3d933eaa1e5190a85d6e028706ab18d262375dd
    22a7c1a0e7efa11851ea29b4c92739aaabfee40353453ece16bda2f4a2c2f86
    e6b37f6de92dc45dba2eb811413c4af2c89f5fc0859718d7cd9888cd8d813da
    2e93726484ea5ce5be8ecf1e1490b874bd897ccd0cbc33db0a1751f81368372
    4b7f5cf750f2497953607d1e82fb5d1429cbfd7a40ccbdba04fb648203c91e0
    809e497e80e9fad7895b844ba6da6ac690c7ce49c10e0000000000000000010
    0ff00000000000000036d2ac8ddbf6eaac95401f91baca7f068e3c237386d7c
    9a271f5187ed909155870200000000e6c8aa48925e37fdab75db13aca7c4f39
    068e12eeb3af8fd1f342005cae5ab9a1ef5344fab2374e9436a67f570418996
    93d333610dfe785d329988736797950d
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style MD013 -->

<!-- markdownlint-disable line-length code-block-style -->
1. Transaction size (u32): `0000037e`
2. Jörmungandr specific tag (u8): `00`
3. Jörmungandr specific tag (u8): `0b`
4. Vote plan id (32 byte hash): `36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b`
5. Proposal index (u8): `00`
6. Payload type tag (u8): `02`
7. Encrypted vote:
   `03|f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|`
   `b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846|`
   `f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|`
   `c8f58976fc0e951ba284a24f3fc190d914ae53aebcc523e7a4a330c8655b4908|`
   `f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|`
   `b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846`
    * size (u8): `03`
    * ciphertext (group element (32 byte), group element (32 byte)):
      `f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|`
      `b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846|`
      `f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|`
      `c8f58976fc0e951ba284a24f3fc190d914ae53aebcc523e7a4a330c8655b4908|`
      `f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|`
      `b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846`
8. Proof:
   `02|1c76d0a50054ef7205cb95c1fd3f928f224fab8a8d70feaf4f5db90630c3845a|`
   `06df2f11c881e396318bd8f9e9f135c2477e923c3decfd6be5466d6166fb3c70|`
   `2edd0d1d0a201fb8c51a91d01328da257971ca78cc566d4b518cb2cd261f9664|`
   `4067a7359a745fe239db8e73059883aece4d506be71c1262b137e295ce5f8a0a|`
   `ac22c1d8d343e5c8b5be652573b85cba8f4dcb46cfa4aafd8d59974e2eb65f48|`
   `0cf85ab522e23203c4f2faa9f95ebc0cd75b04f04fef5d4001d349d1307bb557|`
   `0af4a91d8af4a489297a3f5255c1e12948787271275c50386ab2ef3980d88222|`
   `8e5f3c82d386e6a4ccf7663df5f6bbd9cbbadd6b2fea2668a8bf5603be295461|`
   `52902a35fc44aae80d9dcd85fad6cde5b47a6bdc6257c5937f8de877d5ca0356|`
   `ee9f12a061e03b99ab9dfea56295485cb5ce38cd37f56c396949f58b0627f455|`
   `d26e4c5ff0bc61ab0ff05ffa07880d0e5c540bc45b527e8e85bb1da469935e0d|`
   `3ada75d7d41d785d67d1d0732d7d6cbb12b23bfc21dfb4bbe3d933eaa1e5190a|`
   `85d6e028706ab18d262375dd22a7c1a0e7efa11851ea29b4c92739aaabfee403|`
   `53453ece16bda2f4a2c2f86e6b37f6de92dc45dba2eb811413c4af2c89f5fc08|`
   `59718d7cd9888cd8d813da2e93726484ea5ce5be8ecf1e1490b874bd897ccd0c|`
   `bc33db0a1751f813683724b7f5cf750f2497953607d1e82fb5d1429cbfd7a40c|`
   `cbdba04fb648203c91e0809e497e80e9fad7895b844ba6da6ac690c7ce49c10e`
    * size (u8): `02`
    * announcements (group element (32 byte), group element (32 byte), group element (32 byte)):
      `1c76d0a50054ef7205cb95c1fd3f928f224fab8a8d70feaf4f5db90630c3845a|`
      `06df2f11c881e396318bd8f9e9f135c2477e923c3decfd6be5466d6166fb3c70|`
      `2edd0d1d0a201fb8c51a91d01328da257971ca78cc566d4b518cb2cd261f9664|`
      `4067a7359a745fe239db8e73059883aece4d506be71c1262b137e295ce5f8a0a|`
      `ac22c1d8d343e5c8b5be652573b85cba8f4dcb46cfa4aafd8d59974e2eb65f48|`
      `0cf85ab522e23203c4f2faa9f95ebc0cd75b04f04fef5d4001d349d1307bb557`
    * ciphertext (group element (32 byte), group element (32 byte)):
      `0af4a91d8af4a489297a3f5255c1e12948787271275c50386ab2ef3980d88222|`
      `8e5f3c82d386e6a4ccf7663df5f6bbd9cbbadd6b2fea2668a8bf5603be295461|`
      `52902a35fc44aae80d9dcd85fad6cde5b47a6bdc6257c5937f8de877d5ca0356|`
      `ee9f12a061e03b99ab9dfea56295485cb5ce38cd37f56c396949f58b0627f455`
    * response randomness (scalar (32 byte), scalar (32 byte), scalar (32 byte)):
      `d26e4c5ff0bc61ab0ff05ffa07880d0e5c540bc45b527e8e85bb1da469935e0d|`
      `3ada75d7d41d785d67d1d0732d7d6cbb12b23bfc21dfb4bbe3d933eaa1e5190a|`
      `85d6e028706ab18d262375dd22a7c1a0e7efa11851ea29b4c92739aaabfee403|`
      `53453ece16bda2f4a2c2f86e6b37f6de92dc45dba2eb811413c4af2c89f5fc08|`
      `59718d7cd9888cd8d813da2e93726484ea5ce5be8ecf1e1490b874bd897ccd0c|`
      `bc33db0a1751f813683724b7f5cf750f2497953607d1e82fb5d1429cbfd7a40c`
    * scalar (32 byte): `cbdba04fb648203c91e0809e497e80e9fad7895b844ba6da6ac690c7ce49c10e`
9. `IOW` stand for Inputs-Outputs-Witnesses:
   `00000000000000000100ff00000000000000036d2ac8ddbf6eaac95401f91ba`
   `ca7f068e3c237386d7c9a271f5187ed909155870200000000e6c8aa48925e37`
   `fdab75db13aca7c4f39068e12eeb3af8fd1f342005cae5ab9a1ef5344fab237`
   `4e9436a67f57041899693d333610dfe785d329988736797950d`
    * Jörmungandr specific block date (epoch (u32), slot (u32))
    (*could be anything, not processed anymore*): `00000000|00000000`
    * number of inputs and witnesses (u8) (**always** `1`): `01`
    * number of outputs (u8) (**always** `0`): `00`
    * Inputs
        * Jörmungandr specific tag: `ff`
        * Jörmungandr specific value (u64) (*could be anything, not processed anymore*): `0000000000000003`
        * input pointer (32 byte): `6d2ac8ddbf6eaac95401f91baca7f068e3c237386d7c9a271f5187ed90915587`
    * Witnesses
        * Jörmungandr specific tag (u8): `02`
        * Jörmungandr specific nonce (u32) (*could be anything, not processed anymore*): `00000000`
        * legacy signature (64 byte):
          `e6c8aa48925e37fdab75db13aca7c4f39068e12eeb3af8fd1f342005cae5ab9a1ef5344fab2374e9436a67f57`
          `041899693d333610dfe785d329988736797950d`
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

### Vote generation

To generate a cryptographically secured `ENCRYPTED-VOTE` and `PROOF-VOTE` parts you can follow this [spec](./crypto.md#vote).
Important to note,
that as part of [*initial setup*](./crypto.md#initial-setup) of the voting procedure,
the following properties are used:

1. Each proposal, defined by the `VOTE-PLAN-ID` and `PROPOSAL-INDEX`, defines a number of possible options.
2. [ristretto255] as a backend cryptographic group.
3. A commitment key $ck$ defined as a [BLAKE2b-512] hash of the `VOTE-PLAN-ID` bytes.

### Signing (witness generation)

Signature generated from the [BLAKE2b-256] hashed  `VOTE-PAYLOAD` bytes except of the `WITNESS` part
(the last part from the bytes array):

1. `CAST-CERT`  bytes
2. `BLOCK-DATE` bytes
3. `%x01`
4. `%x00`
5. `INPUT` bytes

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->

??? example "Based on the on the transaction example, data to sign"

    ```hex
    36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7
    b000203f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744
    746c8b6fb0018773d3b4308344d2e90599cd03749658561787eab714b542a5c
    caf078846f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b2869097
    44746c8b6fc8f58976fc0e951ba284a24f3fc190d914ae53aebcc523e7a4a33
    0c8655b4908f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b28690
    9744746c8b6fb0018773d3b4308344d2e90599cd03749658561787eab714b54
    2a5ccaf078846021c76d0a50054ef7205cb95c1fd3f928f224fab8a8d70feaf
    4f5db90630c3845a06df2f11c881e396318bd8f9e9f135c2477e923c3decfd6
    be5466d6166fb3c702edd0d1d0a201fb8c51a91d01328da257971ca78cc566d
    4b518cb2cd261f96644067a7359a745fe239db8e73059883aece4d506be71c1
    262b137e295ce5f8a0aac22c1d8d343e5c8b5be652573b85cba8f4dcb46cfa4
    aafd8d59974e2eb65f480cf85ab522e23203c4f2faa9f95ebc0cd75b04f04fe
    f5d4001d349d1307bb5570af4a91d8af4a489297a3f5255c1e1294878727127
    5c50386ab2ef3980d882228e5f3c82d386e6a4ccf7663df5f6bbd9cbbadd6b2
    fea2668a8bf5603be29546152902a35fc44aae80d9dcd85fad6cde5b47a6bdc
    6257c5937f8de877d5ca0356ee9f12a061e03b99ab9dfea56295485cb5ce38c
    d37f56c396949f58b0627f455d26e4c5ff0bc61ab0ff05ffa07880d0e5c540b
    c45b527e8e85bb1da469935e0d3ada75d7d41d785d67d1d0732d7d6cbb12b23
    bfc21dfb4bbe3d933eaa1e5190a85d6e028706ab18d262375dd22a7c1a0e7ef
    a11851ea29b4c92739aaabfee40353453ece16bda2f4a2c2f86e6b37f6de92d
    c45dba2eb811413c4af2c89f5fc0859718d7cd9888cd8d813da2e93726484ea
    5ce5be8ecf1e1490b874bd897ccd0cbc33db0a1751f813683724b7f5cf750f2
    497953607d1e82fb5d1429cbfd7a40ccbdba04fb648203c91e0809e497e80e9
    fad7895b844ba6da6ac690c7ce49c10e00000000000000000100ff000000000
    00000036d2ac8ddbf6eaac95401f91baca7f068e3c237386d7c9a271f5187ed
    90915587
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

[BLAKE2b-256] hash of the transaction data to sign equals to
`f51473df863be3e0383ce5a8da79c7ff51b3d98dadbbefbf9f042e8601901269`

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->

??? example "Expected witness (includes signature)"

    ```hex
    0200000000e6c8aa48925e37fdab75db13aca7c4f39068e12eeb3af8fd1f342
    005cae5ab9a1ef5344fab2374e9436a67f57041899693d333610dfe785d3299
    88736797950d
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

[ristretto255]: https://ristretto.group/
[BLAKE2b-256]: https://www.blake2.net/blake2.pdf
[BLAKE2b-512]: https://www.blake2.net/blake2.pdf
