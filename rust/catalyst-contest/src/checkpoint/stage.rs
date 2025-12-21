//! The stage of the ballot processing that is represented by a checkpoint.

use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Value for Bulletin Board Stage.
const BULLETIN_BOARD: &str = "bulletin-board";
/// Value for Tally Stage.
const TALLY: &str = "tally";
/// Value for Audit Stage.
const AUDIT: &str = "audit";

/// The stage of the ballot processing that is represented by a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BallotProcessingStage {
    /// Voting is still on-going and ballot box checkpoints are periodically published
    BulletinBoard,
    /// Voting has finished and ballots are collected
    Tally,
    /// Voting has finished and verified.
    Audit,
}

impl Encode<()> for BallotProcessingStage {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let stage_str = match self {
            Self::BulletinBoard => BULLETIN_BOARD,
            Self::Tally => TALLY,
            Self::Audit => AUDIT,
        };
        e.str(stage_str)?;
        Ok(())
    }
}

impl Decode<'_, ()> for BallotProcessingStage {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let stage_str = d.str()?;
        match stage_str {
            BULLETIN_BOARD => Ok(Self::BulletinBoard),
            TALLY => Ok(Self::Tally),
            AUDIT => Ok(Self::Audit),
            _ => {
                Err(minicbor::decode::Error::message(format!(
                    "Invalid stage value: {stage_str}. Expected '{BULLETIN_BOARD}', '{TALLY}', or '{AUDIT}'"
                )))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let stages = [
            BallotProcessingStage::BulletinBoard,
            BallotProcessingStage::Tally,
            BallotProcessingStage::Audit,
        ];

        for original in stages {
            let mut buffer = Vec::new();
            original
                .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
                .unwrap();
            let decoded =
                BallotProcessingStage::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                    .unwrap();
            assert_eq!(original, decoded);
        }
    }
}
