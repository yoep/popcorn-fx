package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.backend.playlists.Playlist;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.controls.SubtitleDropDownButton;
import com.github.yoep.popcorn.ui.view.services.SubtitlePickerService;
import javafx.scene.control.Button;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.Collections;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DetailsTorrentComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private TorrentCollectionService torrentCollectionService;
    @Mock
    private LocaleText localeText;
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private PlaylistManager playlistManager;
    @Mock
    private FxLib fxLib;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private DetailsTorrentComponent component;

    @BeforeEach
    void setUp() {
        component.fileShadow = new Pane();
        component.torrentList = new ListView<>();
        component.subtitleButton = new SubtitleDropDownButton();
        component.playerButton = new PlayerDropDownButton();
        component.storeTorrentButton = new Button();
    }

    @Test
    void testInitialize() {
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn("");
        when(subtitleCustom.getLanguage()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn("");
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);

        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        verify(subtitleService, atLeastOnce()).none();
        assertEquals(subtitleNone, component.subtitleButton.getSelectedItem().get());
    }

    @Test
    void testOnShowTorrentDetailsEvent() throws TimeoutException {
        var filename = " lorem ipsum dolor.mp4";
        var torrent = mock(TorrentInfo.class);
        var fileInfo = mock(TorrentFileInfo.class);
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn("");
        when(subtitleCustom.getLanguage()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn("");
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);
        when(torrent.getFiles()).thenReturn(Collections.singletonList(fileInfo));
        when(fileInfo.getFilename()).thenReturn(filename);
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowTorrentDetailsEvent(this, "", torrent));
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.torrentList.getItems().contains(fileInfo));
    }

    @Test
    void testOnFileInfoClicked() {
        var holder = new AtomicReference<Playlist.ByValue>();
        var torrent = mock(TorrentInfo.class);
        var fileInfo = mock(TorrentFileInfo.class);
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn("");
        when(subtitleCustom.getLanguage()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn("");
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);
        when(torrent.getFiles()).thenReturn(Collections.singletonList(fileInfo));
        doAnswer(invocation -> {
            holder.set(invocation.getArgument(0, Playlist.ByValue.class));
            return null;
        }).when(playlistManager).play(isA(Playlist.ByValue.class));
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowTorrentDetailsEvent(this, "", torrent));
        component.onFileInfoClicked(fileInfo);

        verify(playlistManager).play(isA(Playlist.ByValue.class));
        var result = holder.get().getItems().get(0);
        assertNotNull(result.getTorrentInfo(), "Torrent info should not be null");
        assertNotNull(result.getTorrentFileInfo(), "Torrent file info should not be null");
    }

    @Test
    void testCustomSubtitle() {
        var subtitleFileUri = "/tmp/my-subtitle.srt";
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn("");
        when(subtitleCustom.getLanguage()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn("");
        when(subtitleCustom.isCustom()).thenReturn(true);
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);
        when(subtitlePickerService.pickCustomSubtitle())
                .thenReturn(Optional.of(subtitleFileUri));
        component.initialize(url, resourceBundle);

        component.subtitleButton.select(subtitleService.custom());

        verify(subtitleService).updatePreferredLanguage(SubtitleLanguage.CUSTOM);
        assertEquals(SubtitleLanguage.CUSTOM, component.subtitleInfo.getLanguage());
    }
}