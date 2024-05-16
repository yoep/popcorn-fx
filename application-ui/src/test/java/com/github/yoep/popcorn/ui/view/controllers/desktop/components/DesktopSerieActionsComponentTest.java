package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.media.providers.Episode;
import com.github.yoep.popcorn.backend.media.providers.ShowDetails;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static java.util.Arrays.asList;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopSerieActionsComponentTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private DesktopSerieQualityComponent desktopSerieQualityComponent;
    @Mock
    private PlaylistManager playlistManager;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resources;
    @InjectMocks
    private DesktopSerieActionsComponent component;

    @BeforeEach
    void setUp() {
        component.watchNowButton = new PlayerDropDownButton();
        component.languageSelection = new LanguageFlagSelection();
    }

    @Test
    void testOnWatchNowClicked() {
        var event = mock(MouseEvent.class);
        var show = mock(ShowDetails.class);
        var episode = mock(Episode.class);
        var quality = "1080p";
        when(desktopSerieQualityComponent.getSelectedQuality()).thenReturn(quality);
        when(subtitleService.retrieveSubtitles(show, episode)).thenReturn(new CompletableFuture<>());
        component.initialize(location, resources);

        // update the episode info
        component.episodeChanged(show, episode);
        verify(desktopSerieQualityComponent).episodeChanged(episode);

        component.onWatchNowClicked(event);

        verify(event).consume();
        verify(playlistManager).play(show, episode, quality);
    }

    @Test
    void testOnWatchNowPressed() {
        var event = mock(KeyEvent.class);
        var show = mock(ShowDetails.class);
        var episode = mock(Episode.class);
        var quality = "720p";
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(desktopSerieQualityComponent.getSelectedQuality()).thenReturn(quality);
        when(subtitleService.retrieveSubtitles(show, episode)).thenReturn(new CompletableFuture<>());
        component.initialize(location, resources);

        // update the episode info
        component.episodeChanged(show, episode);
        verify(desktopSerieQualityComponent).episodeChanged(episode);

        component.onWatchNowPressed(event);

        verify(event).consume();
        verify(playlistManager).play(show, episode, quality);
    }

    @Test
    void testOnLanguageChanged() throws TimeoutException {
        var show = mock(ShowDetails.class);
        var episode = mock(Episode.class);
        var none = SubtitleInfo.builder()
                .language(SubtitleLanguage.NONE)
                .build();
        var english = SubtitleInfo.builder()
                .language(SubtitleLanguage.ENGLISH)
                .build();
        var french = SubtitleInfo.builder()
                .language(SubtitleLanguage.FRENCH)
                .build();
        when(subtitleService.retrieveSubtitles(isA(ShowDetails.class), isA(Episode.class))).thenReturn(CompletableFuture.completedFuture(asList(none, english, french)));
        component.initialize(location, resources);

        component.episodeChanged(show, episode);
        WaitForAsyncUtils.waitForFxEvents();
        component.languageSelection.select(french);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.languageSelection.getSelectedItem() == french);
        verify(subtitleService).retrieveSubtitles(show, episode);
        verify(subtitleService).updateSubtitle(french);
    }
}