package com.github.yoep.torrent.frostwire;

import com.github.yoep.torrent.adapter.state.TorrentHealthState;
import com.github.yoep.torrent.frostwire.model.TorrentHealthImpl;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith(MockitoExtension.class)
class TorrentServiceImplTest {
    @Mock
    private TorrentSessionManager sessionManager;
    @Mock
    private TorrentResolverService torrentResolverService;
    @InjectMocks
    private TorrentServiceImpl torrentService;

    @Test
    void testCalculateHealth_whenSeedsIsZeroAndPeersIsZero_shouldReturnUnknown() {
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.UNKNOWN, 0, 0, 0);

        var result = torrentService.calculateHealth(0, 0);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenPeersIsLargerThanSeeds_shouldReturnBad() {
        var seeds = 5;
        var peers = 10;
        var ratio = 0.5;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.BAD, ratio, seeds, peers);

        var result = torrentService.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenSeedsIsEqualToPeers_shouldReturnBad() {
        var seeds = 10;
        var peers = 10;
        var ratio = 1;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.BAD, ratio, seeds, peers);

        var result = torrentService.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenSeedsIsLargerThan30_shouldReturnGood() {
        var seeds = 35;
        var peers = 10;
        var ratio = 3.5;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.GOOD, ratio, seeds, peers);

        var result = torrentService.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }

    @Test
    void testCalculateHealth_whenRatioIsLargerThan5_shouldReturnExcellent() {
        var seeds = 50;
        var peers = 10;
        var ratio = 5;
        var expectedResult = new TorrentHealthImpl(TorrentHealthState.EXCELLENT, ratio, seeds, peers);

        var result = torrentService.calculateHealth(seeds, peers);

        assertEquals(expectedResult, result);
    }
}
