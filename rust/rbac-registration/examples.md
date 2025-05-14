# Examples

## Preface

The goal of this document is to describe how multiple RBAC registrations apply on top of each other with emphasis on
non-obvious cases.
For more general information about RBAC registration and Cip509 see the links in the [readme].

[readme]: https://github.com/input-output-hk/catalyst-libs/blob/main/rust/rbac-registration/README.md

## Glossary

Commonly used abbreviations in this document:

* A stake address or `StakeN` (for example, `Stake1`) - a stake address in the role 0 registration.
* A (public) key or `PubKeyN` (`PubKey3`) - a subject public key of the role 0.
* A chain or `ChainN` (`Chain1`) - a chain of individual registration transactions.
* A root (`Root`) - the first registration in RBAC registration chain.
* A registration or a registration transaction - a transaction that contains CIP509 RBAC registration.

In diagrams registration chains are either marked as `Root --- R1 --- R2 --- ...` when there is a single chain or with
a letter with a number (`A0 --- A1`) when there are multiple ones.

## Examples

### Useless updates

```mermaid
flowchart LR
    Root[Cert] --- R1[Cert] --- R2[Cert] --- R3[...]
    style Root fill: green
    style R1 fill: green
    style R2 fill: green
    style R3 fill: green
```

It is allowed to update the role 0 with the exact same certificate.
While it is completely useless, one can simply change the key every time with the same effect, so it doesn't make sense
to prevent it.

### Invalid registrations

```mermaid
flowchart LR
    Root[Valid Root] --- R1[Valid update 1] --- R2[Invalid update 2] --- R3[Invalid update 3]
    R1 ----- R4[Valid update 4]
    style Root fill: green
    style R1 fill: green
    style R2 fill: red
    style R3 fill: yellow
    style R4 fill: green
```

All following registrations (either a role update or an additional role) must reference the previous valid registration.
In the example above both the root and the first update are valid.
The second update is invalid, so it is ignored and the chain still consists of two transactions.
The third update can be valid by itself, but as it references the previous invalid update it will be discarded too.
The fourth update references the last valid registration (update number 1), so assuming it is valid, the chain now
consists of three registrations (root, update 1 and update 4).

If the root registration is invalid, then it cannot be continued: any registrations that try to reference it as a
previous one will be invalid.

### Multiple registrations in one block

```mermaid
flowchart LR
    Root[Root] --- A[A]
    Root --- B[B]
    Root --- C[C]
    style Root fill: green
    style A fill: yellow
    style B fill: yellow
    style C fill: yellow
```

Technically it is possible for multiple RBAC registration transactions to be included into the same block, but there is
no way to specify the ordering.
Therefore, one should normally wait for the previous transaction to be processed before submitting the next one.

In the example above there are three transactions: `A`, `B` and `C`.
If all of them reference the root registration and are included into the same block then only one of them will be
applied to the registration chain and the other two will be considered invalid.
If these transactions reference each other (`A - B - C`) then the order entirely depends on Cardano blockchain and
cannot be predicted.
It is possible that transactions `B` and `C` would be executed first and rejected as invalid and only `A` will be
applied.
It is better to avoid such situations and submit the next transaction after the previous one was included into a block.

### Branching registrations

```mermaid
flowchart LR
    Root[Slot 100] --- R1[Slot 110] --- R2[Slot 120]
    Root ----- R3[Slot 130] --- R4[Slot 140]
    style Root fill: green
    style R1 fill: green
    style R2 fill: green
    style R3 fill: red
    style R4 fill: yellow
```

Only the latest valid registration in the chain can be referenced.
In the example the registration at slot 130 is ignored because it references the root registration, while the most
recent valid transaction is the one at slot 120.
The registration at slot 140 is also ignored because it references an invalid transaction.

### Updating a stake address

```mermaid
flowchart LR
    Root[Stake1] --- R1[Stake2] --- R2[Stake3]
    style Root fill: green
    style R1 fill: green
    style R2 fill: green
```

It is allowed to change a stake address by updating the role 0.

### Updating a subject public key

```mermaid
flowchart LR
    Root[PubKey1] --- R1[PubKey2] --- R2[PubKey3]
    style Root fill: green
    style R1 fill: green
    style R2 fill: green
```

It is allowed to change a subject public key by updating the role 0.
It is worth noting that the Catalyst ID associated with this registration chain is based on the subject public key of
the very first role 0 registration and updating the key doesn't change the Catalyst ID.
In the example above the Catalyst ID is based on the `PubKey1`.

### Reusing a public key

```mermaid
block-beta
    columns 5
    space s1["slot 1"] s2["slot 2"] s3["slot 3"] s4["slot 4"]
    c1["chain 1"] A0["Stake1\nPubKey1"] A1["Stake1\nPubKey2"] A2["Stake1\nPubKey3"] space
    c2["chain 2"] space space space B0["Stake2\nPubKey1"]
    c3["chain 3"] space space space C0["Stake3\nPubKey2"]
    c4["chain 4"] space space space D0["Stake4\nPubKey3"]
    style A0 fill: green
    style A1 fill: green
    style A2 fill: green
    style B0 fill: red
    style C0 fill: green
    style D0 fill: green
```

It isn't allowed to use the key that was already used to start any of existing chains because it would result in the
same Catalyst ID.
In the example above the `Chain1` chain consists of the initial registration with the `PubKey1` key (therefore the
Catalyst ID is based on it) with two subject public key updates (`PubKey2` and `PubKey3`).
The `Chain2` chain is invalid because it uses the same `PubKey1` and would have the same Catalyst ID.
Both `Chain3` and `Chain4` chains are valid even though they use the `PubKey2` and `PubKey3` keys that are already used
in the `Chain1` chain.

It is worth noting that while it is allowed to reuse a public key there is no practical reason to do so.
The existing chain can be updated instead.

### Restarting a chain

```mermaid
block-beta
    columns 7
    space s1["slot 1"] s2["slot 2"] s3["slot 3"] s4["slot 4"] space res["result"]
    c1["chain 1"] A0["Stake1\nPubKey1"] A1["Stake2\nPubKey2"] space space space AF["\nPubKey2"]
    c2["chain 2"] space space B0["Stake2\nPubKey2"] space space space
    c3["chain 3"] space space space C0["Stake1\nPubKey2"] space CF["Stake1\nPubKey2"]
    c4["chain 4"] space space space D0["Stake2\nPubKey3"] space DF["Stake2\nPubKey3"]
    style A0 fill: green
    style A1 fill: green
    style B0 fill: red
    style C0 fill: green
    style D0 fill: green
    style AF fill: gray
    style CF fill: green
    style DF fill: green
```

In the example above there is the `Chain1` registration chain that was created with some `Stake1` stake address and
`PubKey1` key then both address and key were updated to `Stake2` and `PubKey2`.

The `Chain2` registration is invalid because it uses the same stake address and key.
There are no reason to start a new registration chain with the same data already registered and not to update the
existing one, so this isn't allowed.

The `Chain3` registration is valid because the `PubKey2` key (a Catalyst ID for the new chain would be based on it)
wasn't used before to start a chain and the `Stake1` stake address isn't currently used by `Chain1`.

The `Chain4` registration is valid, but it takes ownership over the `Stake2` stake address of the first chain.
It is allowed because the new chain uses a new public key.
This can be useful if a user lost his private key while maintaining access to his Cardano wallet (and the stake address
used in that registration chain).
This can also be referred to as "restarting" or "overriding" a chain.
After that `Chain1` is still valid and can be potentially continued, but as for now it has zero voting power because it
has no active stake address.
It is allowed to use a RBAC token based on that chain and perform actions such as submitting a proposal.

### Restarting multiple chains

```mermaid
block-beta
    columns 6
    space s1["slot 1"] s2["slot 2"] s3["slot 3"] space res["result"]
    c1["chain 1"] A0["..."] A1["Stake1\nPubKey1"] space space AF["\nPubKey1"]
    c2["chain 2"] B0["..."] B1["Stake2\nPubKey2"] space space BF["\nPubKey2"]
    c3["chain 3"] space space C0["Stake1, Stake2\nPubKey3"] space CF["Stake1, Stake2\nPubKey3"]
    style A0 fill: green
    style A1 fill: green
    style AF fill: gray
    style B0 fill: green
    style B1 fill: green
    style BF fill: gray
    style C0 fill: green
    style CF fill: green
```

It is possible to restart/override multiple chains with one new registration.
In the example above there are two chains with stake addresses `Stake1` and `Stake2` correspondingly.
The new root registration contains both of these addresses, so assuming it is valid, it "restarts" both two previous
registrations.

As explained in the [restarting a chain](#restarting-a-chain) example, a new registration must use a different public
key, so using both `PubKey1` and `PubKey2` isn't allowed.

Both first and second chains are still valid and can be continued, but have zero voting power.

### Restarting chain(s) with multiple stake addresses

```mermaid
block-beta
    columns 6
    space s1["slot 1"] s2["slot 2"] s3["slot 3"] space res["result"]
    c1["chain 1"] A0["..."] A1["Stake1, Stake2\nPubKey1"] space space AF["\nPubKey1"]
    c2["chain 2"] B0["..."] B1["Stake3, Stake4\nPubKey2"] space space BF["Stake4\nPubKey2"]
    c3["chain 3"] space space C0["Stake1, Stake2, Stake3\nPubKey3"] space CF["Stake1, Stake2, Stake3\nPubKey2"]
    style A0 fill: green
    style A1 fill: green
    style AF fill: gray
    style B0 fill: green
    style B1 fill: green
    style BF fill: green
    style C0 fill: green
    style CF fill: green
```

It isn't required for a new chain to use all stake addresses of a previous one.
In the example above the new registration uses only three addresses of four used in two previous chains.
As a result, the first chain has no active stake addresses, but the second one still has the `Stake4` stake address
associated with it.
