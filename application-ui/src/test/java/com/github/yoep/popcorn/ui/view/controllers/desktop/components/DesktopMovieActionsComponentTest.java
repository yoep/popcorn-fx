package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.Images;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.backend.playlists.model.Playlist;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
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
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static java.util.Arrays.asList;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopMovieActionsComponentTest {
    @Mock
    private PlaylistManager playlistManager;
    @Mock
    private PlayerManagerService playerManager;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private DetailsComponentService detailsComponentService;
    @Mock
    private DesktopMovieQualityComponent desktopMovieQualityComponent;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @Mock
    private SubtitleInfo subtitleNone;
    @InjectMocks
    private DesktopMovieActionsComponent component;

    @BeforeEach
    void setUp() {
        lenient().when(subtitleService.retrieveSubtitles(isA(MovieDetails.class))).thenReturn(new CompletableFuture<>());
        lenient().when(subtitleService.none()).thenReturn(subtitleNone);
        lenient().when(subtitleService.custom()).thenReturn(mock(SubtitleInfo.class));
        lenient().when(subtitleNone.language()).thenReturn(SubtitleLanguage.NONE);
        lenient().when(subtitleNone.getFlagResource()).thenReturn("");

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
        var none = SubtitleInfo.builder()
                .imdbId(null)
                .language(SubtitleLanguage.NONE)
                .files(new SubtitleFile[0])
                .build();
        var english = SubtitleInfo.builder()
                .imdbId("tt1122")
                .language(SubtitleLanguage.ENGLISH)
                .files(new SubtitleFile[0])
                .build();
        var german = SubtitleInfo.builder()
                .imdbId("tt1122")
                .language(SubtitleLanguage.GERMAN)
                .files(new SubtitleFile[0])
                .build();
        var media = mock(MovieDetails.class);
        var languages = asList(none, english, german);
        when(subtitleService.retrieveSubtitles(isA(MovieDetails.class))).thenReturn(CompletableFuture.completedFuture(languages));
        when(subtitleService.getDefaultOrInterfaceLanguage(languages)).thenReturn(german);
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.languageSelection.select(english);

        verify(subtitleService).updateSubtitle(english);
    }

    @Test
    void testWatchTrailerClicked() {
        var trailer = "my-movie-trailer";
        var title = "lorem ipsum";
        var event = mock(MouseEvent.class);
        var media = MovieDetails.builder()
                .title(title)
                .trailer(trailer)
                .images(Images.builder().build())
                .build();
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));

        component.onTrailerClicked(event);

        verify(event).consume();
        verify(playlistManager).play(isA(Playlist.class));
        verify(playerManager).addListener(isA(PlayerManagerListener.class));
    }
}