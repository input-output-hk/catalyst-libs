//! Utility functions for CIP-0134 address.

use anyhow::{anyhow, Context, Result};
use pallas::ledger::addresses::Address;

/// Parses CIP-0134 URI and returns an address.
///
/// # Errors
/// - Invalid URI.
///
/// # Examples
///
/// ```
/// use pallas::ledger::addresses::{Address, Network};
/// use rbac_registration::cardano::cip509::utils::parse_cip0134_uri;
///
/// let uri = "web+cardano://addr/stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
/// let Address::Stake(address) = parse_cip0134_uri(uri).unwrap() else {
///     panic!("Unexpected address type");
/// };
/// assert_eq!(address.network(), Network::Mainnet);
/// ```
pub fn parse_cip0134_uri(uri: &str) -> Result<Address> {
    let bech32 = uri
        .strip_prefix("web+cardano://addr/")
        .ok_or_else(|| anyhow!("Missing schema part of URI"))?;
    Address::from_bech32(bech32).context("Unable to parse bech32 part of URI")
}

#[cfg(test)]
mod tests {
    use pallas::ledger::addresses::{Address, Network};

    use super::*;

    #[test]
    fn invalid_prefix() {
        let test_uris = [
            "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
            "//addr/addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
            "web+cardano:/addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
            "somthing+unexpected://addr/addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
        ];

        for uri in test_uris {
            let err = format!("{:?}", parse_cip0134_uri(uri).expect_err(&format!("{uri}")));
            assert_eq!("Missing schema part of URI", err);
        }
    }

    #[test]
    fn invalid_bech32() {
        let uri = "web+cardano://addr/adr1qx2fxv2umyh";
        let err = format!("{:?}", parse_cip0134_uri(uri).unwrap_err());
        assert!(err.starts_with("Unable to parse bech32 part of URI"));
    }

    #[test]
    fn stake_address() {
        let test_data = [
            (
                "web+cardano://addr/stake_test1uqehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gssrtvn",
                Network::Testnet,
                "337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251",
            ),
            (
                "web+cardano://addr/stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw",
                Network::Mainnet,
                "337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251",
            ),
            (
                "web+cardano://addr/drep_vk17axh4sc9zwkpsft3tlgpjemfwc0u5mnld80r85zw7zdqcst6w54sdv4a4e",
                Network::Other(7),
                "4d7ac30513ac1825715fd0196769761fca6e7f69de33d04ef09a0c41",
            )
        ];

        for (uri, network, payload) in test_data {
            let Address::Stake(address) = parse_cip0134_uri(uri).unwrap() else {
                panic!("Unexpected address type ({uri})");
            };
            assert_eq!(network, address.network());
            assert_eq!(payload, address.payload().as_hash().to_string());
        }
    }

    #[test]
    fn shelley_address() {
        let test_data = [
            (
                "web+cardano://addr/addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
                Network::Mainnet,
            ),
            (
                "web+cardano://addr/addr_test1gz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer5pnz75xxcrdw5vky",
                Network::Testnet,
            ),
            (
                "web+cardano://addr/cc_hot_vk10y48lq72hypxraew74lwjjn9e2dscuwphckglh2nrrpkgweqk5hschnzv5",
                Network::Other(9),
            )
        ];

        for (uri, network) in test_data {
            let Address::Shelley(address) = parse_cip0134_uri(uri).unwrap() else {
                panic!("Unexpected address type ({uri})");
            };
            assert_eq!(network, address.network());
        }
    }
}
