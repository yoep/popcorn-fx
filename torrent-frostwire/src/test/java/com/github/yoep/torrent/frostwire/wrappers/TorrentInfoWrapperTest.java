package com.github.yoep.torrent.frostwire.wrappers;

import com.frostwire.jlibtorrent.TorrentInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class TorrentInfoWrapperTest {
    @Mock
    private TorrentInfo torrentInfo;

    @Test
    void testGetNative_whenInvoked_shouldReturnTheNativeHandle() {
        var wrapper = new TorrentInfoWrapper(torrentInfo, "my-torrent-dir");

        var result = wrapper.getNative();

        assertEquals(torrentInfo, result);
    }

    @Test
    void testGetName_whenInvoked_shouldRetrieveTheNameFromTheNativeTorrent() {
        var name = "my-torrent-name";
        var wrapper = new TorrentInfoWrapper(torrentInfo, "my-torrent-dir");
        when(torrentInfo.name()).thenReturn(name);

        var result = wrapper.getName();

        assertEquals(name, result);
    }

    @Test
    void testGetByFilename_whenFilesDoNotIncludeTorrentDirectory_shouldMatchTorrentFileWithoutTorrentDirectory() {
        var filename = "lorem.mp4";
        var torrentDirectoryName = "torrentDirectory";
        var filepath1 = "/lorem.mp4";
        var filepath2 = "/ipsum.mp4";
        var torrentFileInfo1 = mock(TorrentFileInfo.class);
        var torrentFileInfo2 = mock(TorrentFileInfo.class);
        lenient().when(torrentFileInfo1.getFilePath()).thenReturn(filepath1);
        lenient().when(torrentFileInfo2.getFilePath()).thenReturn(filepath2);
        var wrapper = new TorrentInfoWrapper(torrentInfo, torrentDirectoryName, torrentFileInfo1, torrentFileInfo2);

        var result = wrapper.getByFilename(filename);

        assertTrue(result.isPresent(), "Expected torrent file to have been found");
        assertEquals(filepath1, result.get().getFilePath());
    }

    @Test
    void testGetByFilename_whenFilesIncludeTorrentDirectoryName_shouldMatchTorrentFileWithTorrentDirectory() {
        var filename = "ipsum.mp4";
        var torrentDirectoryName = "torrentDirectory";
        var filepath1 = torrentDirectoryName + "/lorem.mp4";
        var filepath2 = torrentDirectoryName + "/ipsum.mp4";
        var torrentFileInfo1 = mock(TorrentFileInfo.class);
        var torrentFileInfo2 = mock(TorrentFileInfo.class);
        lenient().when(torrentFileInfo1.getFilePath()).thenReturn(filepath1);
        lenient().when(torrentFileInfo2.getFilePath()).thenReturn(filepath2);
        var wrapper = new TorrentInfoWrapper(torrentInfo, torrentDirectoryName, torrentFileInfo1, torrentFileInfo2);

        var result = wrapper.getByFilename(filename);

        assertTrue(result.isPresent(), "Expected torrent file to have been found");
        assertEquals(filepath2, result.get().getFilePath());
    }
}