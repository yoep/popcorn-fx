package com.github.yoep.torrent.frostwire.model;

import com.frostwire.jlibtorrent.Priority;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TorrentPiecesTest {
    private TorrentPieces torrentPieces;

    @BeforeEach
    void setUp() {
        torrentPieces = new TorrentPieces();
    }

    @Test
    void testDetermineDownloadPieceIndexes_whenArgumentIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> torrentPieces.determineDownloadPieceIndexes(null), "priorities cannot be null");
    }

    @Test
    void testDetermineDownloadPieceIndexes_whenAllPrioritiesAreIgnored_shouldThrowTorrentException() {
        var pieces = new Priority[]{Priority.IGNORE, Priority.IGNORE, Priority.IGNORE};

        assertThrows(TorrentException.class, () -> torrentPieces.determineDownloadPieceIndexes(pieces),
                "Unable to determine piece indexes, all file priorities are IGNORED");
    }

    @Test
    void testDetermineDownloadPieceIndexes_whenInvoked_shouldDetermineFirstAnLastPieceIndex() {
        var pieces = new Priority[]{Priority.IGNORE, Priority.IGNORE, Priority.NORMAL, Priority.NORMAL, Priority.IGNORE};
        var firstPieceIndex = 2;
        var lastPieceIndex = 3;

        torrentPieces.determineDownloadPieceIndexes(pieces);

        assertEquals(firstPieceIndex, torrentPieces.getFirstPieceIndex());
        assertEquals(lastPieceIndex, torrentPieces.getLastPieceIndex());
    }

    @Test
    void testIsInDownloadRange_whenPieceIsInDownloadRange_shouldReturnTrue() {
        var pieces = new Priority[]{Priority.IGNORE, Priority.IGNORE, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.IGNORE};

        torrentPieces.determineDownloadPieceIndexes(pieces);
        var result = torrentPieces.isInDownloadRange(4);

        assertTrue(result);
    }

    @Test
    void testIsInDownloadRange_whenPieceIsFirstPieceIndex_shouldReturnTrue() {
        var pieces = new Priority[]{Priority.IGNORE, Priority.IGNORE, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.IGNORE};

        torrentPieces.determineDownloadPieceIndexes(pieces);
        var result = torrentPieces.isInDownloadRange(2);

        assertTrue(result);
    }

    @Test
    void testIsInDownloadRange_whenPieceIsLastPieceIndex_shouldReturnTrue() {
        var pieces = new Priority[]{Priority.IGNORE, Priority.IGNORE, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.IGNORE};

        torrentPieces.determineDownloadPieceIndexes(pieces);
        var result = torrentPieces.isInDownloadRange(5);

        assertTrue(result);
    }

    @Test
    void testIsInDownloadRange_whenPieceIsBeforeFirstPieceIndex_shouldReturnFalse() {
        var pieces = new Priority[]{Priority.IGNORE, Priority.IGNORE, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.IGNORE};

        torrentPieces.determineDownloadPieceIndexes(pieces);
        var result = torrentPieces.isInDownloadRange(1);

        assertFalse(result);
    }

    @Test
    void testIsInDownloadRange_whenPieceIsAfterLastPieceIndex_shouldReturnFalse() {
        var pieces = new Priority[]{Priority.IGNORE, Priority.IGNORE, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.NORMAL, Priority.IGNORE};

        torrentPieces.determineDownloadPieceIndexes(pieces);
        var result = torrentPieces.isInDownloadRange(6);

        assertFalse(result);
    }
}
