package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.beans.property.SimpleMapProperty;
import javafx.beans.property.SimpleObjectProperty;
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

import java.net.URL;
import java.util.HashMap;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopSerieActionsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private DesktopSerieQualityComponent desktopSerieQualityComponent;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resources;
    @InjectMocks
    private DesktopSerieActionsComponent component;

    @BeforeEach
    void setUp() {
        when(playerManagerService.playersProperty()).thenReturn(new SimpleMapProperty<>());
        when(playerManagerService.activePlayerProperty()).thenReturn(new SimpleObjectProperty<>());

        component.watchNowButton = new PlayerDropDownButton();
        component.languageSelection = new LanguageFlagSelection();
    }

    @Test
    void testOnWatchNowClicked() {
        var event = mock(MouseEvent.class);
        var show = mock(ShowDetails.class);
        var episode = mock(Episode.class);
        var torrentInfo = mock(MediaTorrentInfo.class);
        var quality = "1080p";
        when(episode.getTorrents()).thenReturn(new HashMap<>() {{
            put(quality, torrentInfo);
        }});
        when(desktopSerieQualityComponent.getSelectedQuality()).thenReturn(quality);
        when(subtitleService.retrieveSubtitles(show, episode)).thenReturn(new CompletableFuture<>());
        component.initialize(location, resources);

        // update the episode info
        component.episodeChanged(show, episode);
        verify(desktopSerieQualityComponent).episodeChanged(episode);

        component.onWatchNowClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new LoadMediaTorrentEvent(component, torrentInfo, show, episode, quality, null));
    }

    @Test
    void testOnWatchNowPressed() {
        var event = mock(KeyEvent.class);
        var show = mock(ShowDetails.class);
        var episode = mock(Episode.class);
        var torrentInfo = mock(MediaTorrentInfo.class);
        var quality = "720p";
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(episode.getTorrents()).thenReturn(new HashMap<>() {{
            put(quality, torrentInfo);
        }});
        when(desktopSerieQualityComponent.getSelectedQuality()).thenReturn(quality);
        when(subtitleService.retrieveSubtitles(show, episode)).thenReturn(new CompletableFuture<>());
        component.initialize(location, resources);

        // update the episode info
        component.episodeChanged(show, episode);
        verify(desktopSerieQualityComponent).episodeChanged(episode);

        component.onWatchNowPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new LoadMediaTorrentEvent(component, torrentInfo, show, episode, quality, null));
    }
}