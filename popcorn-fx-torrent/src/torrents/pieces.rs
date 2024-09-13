use crate::torrents::{InfoHash, PieceError, Result, TorrentError, TorrentInfo};

const METADATA_MISSING_ERR: &str = "missing info metadata";

#[derive(Debug, Clone)]
pub struct Piece {
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct Pieces {
    pub info_hash: InfoHash,
    pub pieces: Vec<Piece>,
}

impl Pieces {
    pub fn total_pieces(&self) -> usize {
        self.pieces.len()
    }
}

impl TryFrom<&TorrentInfo> for Pieces {
    type Error = TorrentError;

    fn try_from(value: &TorrentInfo) -> Result<Self> {
        if let Some(info) = value.info.as_ref() {
            let info_hash = value.info_hash.clone();
            let pieces = info.sha1_pieces();
            let file_size = info.total_size();
            let piece_length = info.piece_length as usize;

            if piece_length == 0 {
                return Err(TorrentError::Piece(PieceError::UnableToDeterminePieces(
                    "invalid piece length 0".to_string(),
                )));
            }

            let num_pieces = (file_size + piece_length - 1) / piece_length;

            if pieces.len() != num_pieces {
                return Err(TorrentError::Piece(PieceError::UnableToDeterminePieces(
                    format!(
                        "calculated pieces {} do not match the expected hash pieces {}",
                        num_pieces,
                        pieces.len()
                    ),
                )));
            }

            let mut last_piece_length = file_size % piece_length;
            if last_piece_length == 0 {
                last_piece_length = piece_length;
            }

            let mut pieces = vec![];

            for i in 0..num_pieces {
                let length = if i == num_pieces - 1 {
                    last_piece_length
                } else {
                    piece_length
                };

                pieces.push(Piece { length })
            }

            return Ok(Self { info_hash, pieces });
        }

        Err(TorrentError::InvalidMetadata(
            METADATA_MISSING_ERR.to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use super::*;

    #[test]
    fn test_try_from() {
        init_logger();
        let info_bytes = read_test_file_to_bytes("debian.torrent");
        let torrent_info = TorrentInfo::try_from(info_bytes.as_slice()).unwrap();
        let expected_total_pieces = torrent_info.info.as_ref().unwrap().pieces.len() / 20;

        let result = Pieces::try_from(&torrent_info).expect("expected the pieces to be created");

        assert_eq!(torrent_info.info_hash, result.info_hash);
        assert_eq!(expected_total_pieces, result.total_pieces());
    }

    #[test]
    fn test_try_from_info_missing() {
        init_logger();
        let torrent_info = TorrentInfo::builder()
            .info_hash(
                InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap(),
            )
            .build();

        let result = Pieces::try_from(&torrent_info);

        match result {
            Err(e) => assert_eq!(
                TorrentError::InvalidMetadata(METADATA_MISSING_ERR.to_string()),
                e
            ),
            _ => assert!(false, "expected Err, but got {:?} instead", result),
        }
    }
}
