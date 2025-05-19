package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.backend.torrent.TorrentCollectionService;
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
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
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
    private ISubtitleService subtitleService;
    @Mock
    private PlaylistManager playlistManager;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private DetailsTorrentComponent component;

    @BeforeEach
    void setUp() {
        when(playerManagerService.getPlayers()).thenReturn(CompletableFuture.completedFuture(Collections.emptyList()));
        when(playerManagerService.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.empty()));

        component.fileShadow = new Pane();
        component.torrentList = new ListView<>();
        component.subtitleButton = new SubtitleDropDownButton();
        component.playerButton = new PlayerDropDownButton();
        component.storeTorrentButton = new Button();
    }

    @Test
    void testInitialize() {
        List<ISubtitleInfo> defaultSubtitles = asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        );
        when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(defaultSubtitles));

        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        verify(subtitleService, atLeastOnce()).defaultSubtitles();
        assertEquals(defaultSubtitles.getFirst(), component.subtitleButton.getSelectedItem().get());
    }

    @Test
    void testOnShowTorrentDetailsEvent() throws TimeoutException {
        var filename = " lorem ipsum dolor.mp4";
        var torrentFile = Torrent.Info.File.newBuilder()
                .setFilename(filename)
                .build();
        var torrent = Torrent.Info.newBuilder()
                .addFiles(torrentFile)
                .build();
        List<ISubtitleInfo> defaultSubtitles = asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        );
        when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(defaultSubtitles));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        eventPublisher.publish(new ShowTorrentDetailsEvent(this, torrent));
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.torrentList.getItems().contains(torrentFile));
    }

    @Test
    void testOnFileInfoClicked() {
        var holder = new AtomicReference<Playlist>();
        var url = "magnet:?xt=urn:btih:Example";
        var filename = "MyVideoFilename.mp4";
        var torrent = Torrent.Info.newBuilder()
                .setUri(url)
                .addFiles(Torrent.Info.File.newBuilder()
                        .setFilename(filename)
                        .build())
                .build();
        List<ISubtitleInfo> defaultSubtitles = asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        );
        when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(defaultSubtitles));
        when(torrentCollectionService.isStored(isA(String.class))).thenReturn(CompletableFuture.completedFuture(true));
        doAnswer(invocation -> {
            holder.set(invocation.getArgument(0, Playlist.class));
            return null;
        }).when(playlistManager).play(isA(Playlist.class));
        component.initialize(this.url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        eventPublisher.publish(new ShowTorrentDetailsEvent(this, torrent));
        component.onFileInfoClicked(torrent.getFilesList().getFirst());

        verify(playlistManager).play(isA(Playlist.class));
        var result = holder.get().getItemsList().getFirst();
        assertEquals(url, result.getUrl(), "url should match torrent magnet url");
        assertEquals(filename, result.getTorrentFilename(), "filename should match the selected filename");
    }

    @Test
    void testCustomSubtitle() {
        var subtitleFileUri = "/tmp/my-subtitle.srt";
        List<ISubtitleInfo> defaultSubtitles = asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        );
        when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(defaultSubtitles));
        when(subtitlePickerService.pickCustomSubtitle()).thenReturn(Optional.of(subtitleFileUri));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.subtitleButton.select(defaultSubtitles.get(1));

        verify(subtitleService).updatePreferredLanguage(Subtitle.Language.CUSTOM);
        assertEquals(Subtitle.Language.CUSTOM, component.subtitleInfo.getLanguage());
    }
}