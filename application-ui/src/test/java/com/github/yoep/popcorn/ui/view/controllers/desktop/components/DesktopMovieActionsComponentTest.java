package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.view.controls.LanguageFlagSelection;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableMap;
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
import org.springframework.core.io.Resource;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopMovieActionsComponentTest {
    @Mock
    private PlayerManagerService playerService;
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

    private final ObservableMap<String, Player> playerProperty = FXCollections.observableMap(new LinkedHashMap<>());
    private final ObjectProperty<Player> activePlayerProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        when(playerService.playersProperty()).thenReturn(playerProperty);
        when(playerService.activePlayerProperty()).thenReturn(activePlayerProperty);
        lenient().when(subtitleService.retrieveSubtitles(isA(MovieDetails.class))).thenReturn(new CompletableFuture<>());
        lenient().when(subtitleService.none()).thenReturn(subtitleNone);
        lenient().when(subtitleService.custom()).thenReturn(mock(SubtitleInfo.class));
        lenient().when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        lenient().when(subtitleNone.getFlagResource()).thenReturn(mock(Resource.class));

        component.watchNowButton = new PlayerDropDownButton();
        component.watchTrailerButton = new Button();
        component.languageSelection = new LanguageFlagSelection();
    }

    @Test
    void testWatchNowClicked() {
        var event = mock(MouseEvent.class);
        var media = mock(MovieDetails.class);
        when(media.getTorrents()).thenReturn(getTorrents());
        when(desktopMovieQualityComponent.getSelectedQuality()).thenReturn("720p");
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onWatchNowClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(isA(LoadMediaTorrentEvent.class));
    }

    @Test
    void testWatchNowPressed() {
        var event = mock(KeyEvent.class);
        var media = mock(MovieDetails.class);
        when(media.getTorrents()).thenReturn(getTorrents());
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(desktopMovieQualityComponent.getSelectedQuality()).thenReturn("720p");
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onWatchNowPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(isA(LoadMediaTorrentEvent.class));
    }

    @Test
    void testWatchTrailerClicked() {
        var trailer = "my-movie-trailer";
        var title = "lorem ipsum";
        var event = mock(MouseEvent.class);
        var media = mock(MovieDetails.class);
        when(media.getTrailer()).thenReturn(trailer);
        when(media.getTitle()).thenReturn(title);
        when(media.getImages()).thenReturn(mock(Images.class));
        component.initialize(url, resourceBundle);
        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));

        component.onTrailerClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(PlayVideoEvent.builder()
                .source(component)
                .url(trailer)
                .title(title)
                .subtitlesEnabled(false)
                .build());
    }

    private Map<String, Map<String, MediaTorrentInfo>> getTorrents() {
        return new HashMap<>() {{
            put(DesktopMovieActionsComponent.DEFAULT_TORRENT_AUDIO, new HashMap<>() {{
                put("480p", mock(MediaTorrentInfo.class));
                put("720p", mock(MediaTorrentInfo.class));
                put("1080p", mock(MediaTorrentInfo.class));
            }});
        }};
    }
}