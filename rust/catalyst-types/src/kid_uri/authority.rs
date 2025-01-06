//! COSE Signature Protected Header `kid` URI Authority.

use std::{
  fmt::{Display, Formatter},
  str::FromStr,
};

/// URI Authority
#[derive(Debug, Clone)]
pub enum Authority {
  /// Cardano Blockchain
  Cardano,
  /// Midnight Blockchain
  Midnight,
}

impl FromStr for Authority {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
      match s {
          "cardano" => Ok(Authority::Cardano),
          "midnight" => Ok(Authority::Midnight),
          _ => Err(anyhow::anyhow!("Unknown Authority: {s}")),
      }
  }
}

impl Display for Authority {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
      let authority = match self {
          Self::Cardano => "cardano",
          Self::Midnight => "midnight",
      };
      write!(f, "{authority}")
  }
}