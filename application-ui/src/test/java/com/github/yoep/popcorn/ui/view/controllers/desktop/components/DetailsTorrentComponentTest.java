package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.playlists.model.Playlist;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile;
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
        var subtitleNone = SubtitleInfo.builder()
                .language(SubtitleLanguage.NONE)
                .files(new SubtitleFile[0])
                .build();
        var subtitleCustom = SubtitleInfo.builder()
                .language(SubtitleLanguage.CUSTOM)
                .files(new SubtitleFile[0])
                .build();
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
        when(subtitleNone.language()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn("");
        when(subtitleCustom.language()).thenReturn(SubtitleLanguage.CUSTOM);
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
        var holder = new AtomicReference<Playlist>();
        var torrent = mock(TorrentInfo.class);
        var fileInfo = mock(TorrentFileInfo.class);
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        var url = "magnet:?xt=urn:btih:Example";
        var filename = "MyVideoFilename.mp4";
        when(subtitleNone.language()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn("");
        when(subtitleCustom.language()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn("");
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);
        when(torrent.getMagnetUri()).thenReturn(url);
        when(torrent.getFiles()).thenReturn(Collections.singletonList(fileInfo));
        when(fileInfo.getFilename()).thenReturn(filename);
        doAnswer(invocation -> {
            holder.set(invocation.getArgument(0, Playlist.class));
            return null;
        }).when(playlistManager).play(isA(Playlist.class));
        component.initialize(this.url, resourceBundle);

        eventPublisher.publish(new ShowTorrentDetailsEvent(this, "", torrent));
        component.onFileInfoClicked(fileInfo);

        verify(playlistManager).play(isA(Playlist.class));
        var result = holder.get().items().get(0);
        assertEquals(url, result.url(), "url should match torrent magnet url");
        assertEquals(filename, result.torrentFilename(), "filename should match the selected filename");
    }

    @Test
    void testCustomSubtitle() {
        var subtitleFileUri = "/tmp/my-subtitle.srt";
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.language()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn("");
        when(subtitleCustom.language()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn("");
        when(subtitleCustom.isCustom()).thenReturn(true);
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);
        when(subtitlePickerService.pickCustomSubtitle())
                .thenReturn(Optional.of(subtitleFileUri));
        component.initialize(url, resourceBundle);

        component.subtitleButton.select(subtitleService.custom());

        verify(subtitleService).updatePreferredLanguage(SubtitleLanguage.CUSTOM);
        assertEquals(SubtitleLanguage.CUSTOM, component.subtitleInfo.language());
    }
}