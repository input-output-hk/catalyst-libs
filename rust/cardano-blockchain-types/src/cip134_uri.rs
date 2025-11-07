//! An URI in the CIP-0134 format.

// Ignore URIs that are used in tests and doc-examples.
// cSpell:ignoreRegExp web\+cardano:.+

use std::fmt::{Display, Formatter};

use anyhow::{Context, Error, Result, anyhow};
use pallas_addresses::Address;

/// A URI in the CIP-0134 format.
///
/// See the [proposal] for more details.
///
/// [proposal]: https://github.com/cardano-foundation/CIPs/pull/888
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct Cip0134Uri {
    /// A URI string.
    uri: String,
    /// An address parsed from the URI.
    address: Address,
}

impl Cip0134Uri {
    /// Creates a new `Cip0134Uri` instance by parsing the given URI.
    ///
    /// # Errors
    /// - Invalid URI.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Cip0134Uri;
    ///
    /// let uri = "web+cardano://addr/stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
    /// let cip0134_uri = Cip0134Uri::parse(uri).unwrap();
    /// ```
    pub fn parse(uri: &str) -> Result<Self> {
        let bech32 = uri
            .strip_prefix("web+cardano://addr/")
            .ok_or_else(|| anyhow!("Missing schema part of URI"))?;
        let address = Address::from_bech32(bech32).context("Unable to parse bech32 part of URI")?;

        Ok(Self {
            uri: uri.to_owned(),
            address,
        })
    }

    /// Returns a URI string.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Cip0134Uri;
    ///
    /// let uri = "web+cardano://addr/stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
    /// let cip0134_uri = Cip0134Uri::parse(uri).unwrap();
    /// assert_eq!(cip0134_uri.uri(), uri);
    /// ```
    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns a URI string.
    ///
    /// # Examples
    ///
    /// ```
    /// use cardano_blockchain_types::Cip0134Uri;
    /// use pallas_addresses::{Address, Network};
    ///
    /// let uri = "web+cardano://addr/stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
    /// let cip0134_uri = Cip0134Uri::parse(uri).unwrap();
    /// let Address::Stake(address) = cip0134_uri.address() else {
    ///     panic!("Unexpected address type");
    /// };
    /// assert_eq!(address.network(), Network::Mainnet);
    /// ```
    #[must_use]
    pub fn address(&self) -> &Address {
        &self.address
    }
}

impl Display for Cip0134Uri {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.uri())
    }
}

impl TryFrom<&[u8]> for Cip0134Uri {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let address = std::str::from_utf8(value)
            .with_context(|| format!("Invalid utf8 string: '{value:?}'"))?;
        Self::parse(address)
    }
}

#[cfg(test)]
mod tests {
    use pallas_addresses::{Address, Network};

    use super::*;

    #[test]
    fn invalid_prefix() {
        // cSpell:disable
        let test_uris = [
            "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
            "//addr/addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
            "web+cardano:/addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
            "somthing+unexpected://addr/addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
        ];
        // cSpell:enable

        for uri in test_uris {
            let err = format!("{:?}", Cip0134Uri::parse(uri).expect_err(uri));
            assert!(err.starts_with("Missing schema part of URI"));
        }
    }

    #[test]
    fn invalid_bech32() {
        let uri = "web+cardano://addr/adr1qx2fxv2umyh";
        let err = format!("{:?}", Cip0134Uri::parse(uri).unwrap_err());
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
            ),
        ];

        for (uri, network, payload) in test_data {
            let cip0134_uri = Cip0134Uri::parse(uri).expect(uri);
            let Address::Stake(address) = cip0134_uri.address() else {
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
            ),
        ];

        for (uri, network) in test_data {
            let cip0134_uri = Cip0134Uri::parse(uri).expect(uri);
            let Address::Shelley(address) = cip0134_uri.address() else {
                panic!("Unexpected address type ({uri})");
            };
            assert_eq!(network, address.network());
        }
    }

    // The Display should return the original URI.
    #[test]
    fn display() {
        let uri = "web+cardano://addr/stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
        let cip0134_uri = Cip0134Uri::parse(uri).expect(uri);
        assert_eq!(uri, cip0134_uri.to_string());
    }
}
