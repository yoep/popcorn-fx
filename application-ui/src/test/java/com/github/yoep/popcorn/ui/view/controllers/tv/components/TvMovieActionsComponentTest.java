package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.Images;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
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
import java.util.HashMap;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvMovieActionsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private LocaleText localeText;
    @Mock
    private DetailsComponentService detailsComponentService;
    @Mock
    private VideoQualityService videoQualityService;
    @Mock
    private PlaylistManager playlistManager;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resources;
    @InjectMocks
    private TvMovieActionsComponent component;

    private final AtomicReference<DetailsComponentListener> listener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        var none = mock(SubtitleInfo.class);
        when(none.language()).thenReturn(SubtitleLanguage.NONE);
        lenient().when(subtitleService.none()).thenReturn(none);
        doAnswer(invocation -> {
            listener.set(invocation.getArgument(0));
            return null;
        }).when(detailsComponentService).addListener(isA(DetailsComponentListener.class));

        component.watchNowButton = new Button();
        component.watchTrailerButton = new Button();
        component.favoriteButton = new Button();
        component.favoriteIcon = new Icon();
        component.qualities = new AxisItemSelection<>();
        component.qualityOverlay = new Overlay();
        component.subtitles = new AxisItemSelection<>();
        component.subtitleOverlay = new Overlay();
    }

    @Test
    void testOnLikeStateChangedToLiked() throws TimeoutException {
        var expectedText = "remove";
        var imdbId = "tt11111";
        var media = mock(MovieDetails.class);
        when(media.getImdbId()).thenReturn(imdbId);
        when(localeText.get(DetailsMessage.REMOVE)).thenReturn(expectedText);
        when(detailsComponentService.isLiked(media)).thenReturn(true);
        mockTorrents(media);
        mockSubtitles(media);
        component.initialize(location, resources);
        eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, media));
        var listener = this.listener.get();

        listener.onLikedChanged(imdbId, true);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_UNICODE));
        assertEquals(expectedText, component.favoriteButton.getText());
    }

    @Test
    void testOnLikeStateChangedToUnliked() throws TimeoutException {
        var expectedText = "add";
        var imdbId = "tt11111";
        var media = mock(MovieDetails.class);
        when(media.getImdbId()).thenReturn(imdbId);
        when(localeText.get(DetailsMessage.ADD)).thenReturn(expectedText);
        when(detailsComponentService.isLiked(media)).thenReturn(false);
        mockTorrents(media);
        mockSubtitles(media);
        component.initialize(location, resources);
        eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, media));
        var listener = this.listener.get();

        listener.onLikedChanged(imdbId, true);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.favoriteIcon.getText().equals(Icon.HEART_O_UNICODE));
        assertEquals(expectedText, component.favoriteButton.getText());
    }

    @Test
    void testOnFavoriteClicked() {
        var event = mock(MouseEvent.class);
        var media = mock(MovieDetails.class);
        mockTorrents(media);
        mockSubtitles(media);
        component.initialize(location, resources);
        eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onFavoriteClicked(event);

        verify(event).consume();
        verify(detailsComponentService).toggleLikedState(media);
    }

    @Test
    void testOnFavoriteKeyPressed() {
        var event = mock(KeyEvent.class);
        var media = mock(MovieDetails.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        mockTorrents(media);
        mockSubtitles(media);
        component.initialize(location, resources);
        eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, media));
        WaitForAsyncUtils.waitForFxEvents();

        component.onFavoritePressed(event);

        verify(event).consume();
        verify(detailsComponentService).toggleLikedState(media);
    }

    @Test
    void testOnSubtitleItemActivated() {
        var qualityNode = new Button();
        var subtitleNode = new Button();
        var subtitle_info = mock(SubtitleInfo.class);
        var movie = MovieDetails.builder()
                .images(Images.builder().build())
                .build();
        var quality = "720p";
        var qualityEvent = mock(MouseEvent.class);
        var subtitleEvent = mock(MouseEvent.class);
        when(qualityEvent.getSource()).thenReturn(qualityNode);
        when(subtitleEvent.getSource()).thenReturn(subtitleNode);
        when(subtitleService.retrieveSubtitles(isA(MovieDetails.class))).thenReturn(new CompletableFuture<>());
        component.initialize(location, resources);

        eventPublisher.publish(new ShowMovieDetailsEvent(this, movie));
        WaitForAsyncUtils.waitForFxEvents();

        component.qualities.setItemFactory(e -> {
            qualityNode.setText(e);
            return qualityNode;
        });
        component.qualities.add(quality);
        qualityNode.getOnMouseClicked().handle(qualityEvent);
        WaitForAsyncUtils.waitForFxEvents();

        component.subtitles.setItemFactory(info -> {
            subtitleNode.setText("Lorem");
            if (info == subtitle_info) {
                return subtitleNode;
            } else {
                return new Button("Foo");
            }
        });
        component.subtitles.add(subtitle_info);
        subtitleNode.getOnMouseClicked().handle(subtitleEvent);
        WaitForAsyncUtils.waitForFxEvents();

        verify(playlistManager).play(movie, quality);
        verify(subtitleService).retrieveSubtitles(movie);
    }

    private void mockSubtitles(MovieDetails media) {
        when(subtitleService.retrieveSubtitles(media)).thenReturn(new CompletableFuture<>());
    }

    private static void mockTorrents(MovieDetails media) {
        when(media.getTorrents()).thenReturn(new HashMap<>() {{
            put(TvMovieActionsComponent.DEFAULT_TORRENT_AUDIO, new HashMap<>());
        }});
    }
}