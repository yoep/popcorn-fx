package com.github.yoep.torrent.frostwire.listeners;

import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.TorrentAlert;
import com.frostwire.jlibtorrent.swig.add_torrent_alert;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.function.Consumer;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
public class TorrentCreationListenerTest {
    @Mock
    private Consumer<TorrentHandle> onCompletionConsumer;
    @Mock
    private TorrentAlert<add_torrent_alert> alert;

    @Test
    void testAlert_whenNameDoesNotMatchExpectedName_shouldNotCallOnCompletion() {
        var creationListener = new TorrentCreationListener("ipsum", onCompletionConsumer);
        when(alert.torrentName()).thenReturn("lorem");

        creationListener.alert(alert);

        verify(onCompletionConsumer, times(0)).accept(any());
    }

    @Test
    void testAlert_whenNameMatchesExpectedName_shouldCallOnCompletion() {
        var name = "ipsum";
        var creationListener = new TorrentCreationListener(name, onCompletionConsumer);
        when(alert.torrentName()).thenReturn(name);

        creationListener.alert(alert);

        verify(onCompletionConsumer).accept(any());
    }
}
