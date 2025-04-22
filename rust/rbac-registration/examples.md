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
* A root - the first registration in RBAC registration chain.

In diagrams registration chains are either marked as `Root --- R1 --- R2 --- ...` when there is a single chain or with
a letter with a number (`A0 --- A1`) when there are multiple ones.

## Examples

### Useless updates

```mermaid
flowchart LR
    Root[Cert] --- R1[Cert] --- R2[Cert] --- R3[...]
```

It is allowed to update the role 0 with the exact same certificate.
While it is completely useless, one can simply change the key every time with the same effect, so it doesn't make sense
to prevent it.

### Updating a stake address

```mermaid
flowchart LR
    Root[Stake1] --- R1[Stake2] --- R2[Stake3]
```

It is allowed to change a stake address by updating the role 0.

### Updating a public key

```mermaid
flowchart LR
    Root[PubKey1] --- R1[PubKey2] --- R2[PubKey3]
```

It is allowed to change a public key by updating the role 0.
It is worth noting that the Catalyst ID associated with this registration chain is based on the subject public key of
the very first role 0 registration and updating the key doesn't change the Catalyst ID.
In the example above the Catalyst ID is based on the `PubKey1`.

### Reusing a public key

```mermaid
flowchart LR
    subgraph Chain4
        direction LR
        E0[Stake4 PubKey3]
    end
    subgraph Chain3
        direction LR
        D0[Stake3 PubKey2]
    end
    subgraph Chain2
        direction LR
        B0[Stake2 PubKey1]
    end
    subgraph Chain1
        direction LR
        A0[Stake1 PubKey1] --- A1[Stake1 PubKey2] --- A2[Stake1 PubKey3]
    end
```

It isn't allowed to use the key that was already used to start any of existing chains because it would result in the
same Catalyst ID.
In the example above the `Chain1` chain consists of the initial registration with the `PubKey1` key (therefore the
Catalyst ID is based on it) and two updated with `PubKey2` and `PubKey3`.
The `Chain2` chain is invalid because it uses the same `PubKey1` and would have the same Catalyst ID.
Both `Chain3` and `Chain4` chains are valid even though they use the `PubKey2` and `PubKey3` keys that are already used
in the `Chain1` chain.

It is worth noting that while it is allowed to reuse a public key there is no practical reason to do so.
The existing chain can be updated instead.

### Restarting a chain

```mermaid
flowchart LR
    subgraph Chain4
        direction LR
        E0[Stake2 PubKey3]
    end
    subgraph Chain3
        direction LR
        D0[Stake1 PubKey2]
    end
    subgraph Chain2
        direction LR
        B0[Stake2 PubKey2]
    end
    subgraph Chain1
        direction LR
        A0[Stake1 PubKey1] --- A1[Stake2 PubKey2]
    end
```

In the example above there is the `Chain1` registration chain that was created with some `Stake1` stake address and
`PubKey1` key then both address and key were updated to `Stake2` and `PubKey2`.
The `Chain2` registration is invalid because it uses the same stake address and key.
There are no reason to start a new registration chain and not to update the existing one, so this isn't allowed.

The `Chain3` registration is valid because the `PubKey2` key wasn't used before to start a chain and the `Stake1` stake
address isn't currently used by `Chain1`.
Warning: this scenario is currently problematic because we never remove entries from the `catalyst_id_for_stake_address`
table.
Therefore, when registration information is requested using the `Stake1` address we need to build both chains to
determine that only `Chain3` is relevant.

The `Chain4` registration is valid, but it "overrides" `Chain1` essentially discarding it and starting a new chain.
This can be useful if a user lost his private key while maintaining access to his Cardano wallet (and the stake address
used in that registration chain).
After that `Chain1` is considered no longer valid and must not be used.

### Multiple stake addresses

Multiple stake addresses support will be addressed in a future revision.
Currently we only support a single stake address per registration chain.
