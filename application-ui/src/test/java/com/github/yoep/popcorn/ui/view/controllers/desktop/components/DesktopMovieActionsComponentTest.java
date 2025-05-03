package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
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
import java.util.concurrent.CompletableFuture;

import static java.util.Arrays.asList;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopMovieActionsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private PlaylistManager playlistManager;
    @Mock
    private PlayerManagerService playerManager;
    @Mock
    private LocaleText localeText;
    @Mock
    private ISubtitleService subtitleService;
    @Mock
    private DetailsComponentService detailsComponentService;
    @Mock
    private DesktopMovieQualityComponent desktopMovieQualityComponent;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private DesktopMovieActionsComponent component;

    @BeforeEach
    void setUp() {
        lenient().when(subtitleService.retrieveSubtitles(isA(MovieDetails.class))).thenReturn(new CompletableFuture<>());
        lenient().when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        )));
        when(playerManager.getPlayers()).thenReturn(CompletableFuture.completedFuture(Collections.emptyList()));
        when(playerManager.getActivePlayer()).thenReturn(CompletableFuture.completedFuture(Optional.empty()));

        component.watchNowButton = new PlayerDropDownButton();
        component.watchTrailerButton = new Button();
        component.languageSelection = new LanguageFlagSelection();
    }

    @Test
    void testWatchNowClicked() {
        var event = mock(MouseEvent.class);
        var media = mock(MovieDetails.class);
        var quality = "720p";
        when(desktopMovieQualityComponent.getSelectedQuality()).thenReturn(quality);
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onWatchNowClicked(event);

        verify(event).consume();
        verify(playlistManager).play(media, quality);
        verify(playerManager).addListener(isA(PlayerManagerListener.class));
    }

    @Test
    void testWatchNowPressed() {
        var event = mock(KeyEvent.class);
        var media = mock(MovieDetails.class);
        var quality = "720p";
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(desktopMovieQualityComponent.getSelectedQuality()).thenReturn(quality);
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onWatchNowPressed(event);

        verify(event).consume();
        verify(playlistManager).play(media, quality);
        verify(playerManager).addListener(isA(PlayerManagerListener.class));
    }

    @Test
    void testLanguageSelectionChanged() {
        var english = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setImdbId("tt1122")
                .setLanguage(Subtitle.Language.ENGLISH)
                .build());
        var german = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setImdbId("tt1122")
                .setLanguage(Subtitle.Language.GERMAN)
                .build());
        var media = new MovieDetails(Media.MovieDetails.newBuilder()
                .setImdbId("tt100000")
                .build());
        when(subtitleService.retrieveSubtitles(media)).thenReturn(CompletableFuture.completedFuture(asList(
                english, german
        )));
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.languageSelection.select(english);
        WaitForAsyncUtils.waitForFxEvents();

        verify(subtitleService).retrieveSubtitles(media);
        verify(subtitleService).updatePreferredLanguage(english.proto().getLanguage());
    }

    @Test
    void testWatchTrailerClicked() {
        var trailer = "my-movie-trailer";
        var title = "lorem ipsum";
        var event = mock(MouseEvent.class);
        var media = new MovieDetails(Media.MovieDetails.newBuilder()
                .setTitle(title)
                .setTrailer(trailer)
                .setImages(Media.Images.newBuilder()
                        .setPoster("http://localhost:8076/poster.jpg")
                        .build())
                .build());
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));

        component.onTrailerClicked(event);

        verify(event).consume();
        verify(playlistManager).play(isA(Playlist.class));
        verify(playerManager).addListener(isA(PlayerManagerListener.class));
    }
}