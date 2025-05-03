package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
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
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvMovieActionsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ISubtitleService subtitleService;
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
    private TvMovieActionsComponent component;

    private final AtomicReference<DetailsComponentListener> listener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().when(subtitleService.defaultSubtitles()).thenReturn(CompletableFuture.completedFuture(asList(
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build()),
                new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build())
        )));
        lenient().when(subtitleService.getDefaultOrInterfaceLanguage(isA(List.class)))
                .thenReturn(CompletableFuture.completedFuture(new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build())));

        doAnswer(invocation -> {
            listener.set(invocation.getArgument(0));
            return null;
        }).when(detailsComponentService).addListener(isA(DetailsComponentListener.class));

        component = new TvMovieActionsComponent(eventPublisher, subtitleService, videoQualityService, localeText, detailsComponentService, playlistManager);

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
        var media = new MovieDetails(Media.MovieDetails.newBuilder()
                .setImdbId(imdbId)
                .setTitle("MyMovie")
                .build());
        when(localeText.get(DetailsMessage.REMOVE)).thenReturn(expectedText);
        when(detailsComponentService.isLiked(media)).thenReturn(CompletableFuture.completedFuture(true));
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
        var media = new MovieDetails(Media.MovieDetails.newBuilder()
                .setImdbId(imdbId)
                .setTitle("MyMovie")
                .build());
        when(localeText.get(DetailsMessage.ADD)).thenReturn(expectedText);
        when(detailsComponentService.isLiked(media)).thenReturn(CompletableFuture.completedFuture(false));
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
        var media = new MovieDetails(Media.MovieDetails.newBuilder()
                .setImdbId("tt000002")
                .setTitle("Foo")
                .build());
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(detailsComponentService.isLiked(media)).thenReturn(CompletableFuture.completedFuture(true));
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
        var subtitleInfo = mock(ISubtitleInfo.class);
        var media = new MovieDetails(Media.MovieDetails.newBuilder()
                .setImdbId("tt00001")
                .setTitle("MyMovie")
                .build());
        var quality = "720p";
        var qualityEvent = mock(MouseEvent.class);
        var subtitleEvent = mock(MouseEvent.class);
        when(qualityEvent.getSource()).thenReturn(qualityNode);
        when(subtitleEvent.getSource()).thenReturn(subtitleNode);
        when(subtitleService.retrieveSubtitles(isA(MovieDetails.class))).thenReturn(new CompletableFuture<>());
        when(detailsComponentService.isLiked(media)).thenReturn(CompletableFuture.completedFuture(true));
        component.initialize(location, resources);

        eventPublisher.publish(new ShowMovieDetailsEvent(this, media));
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
            if (info == subtitleInfo) {
                return subtitleNode;
            } else {
                return new Button("Foo");
            }
        });
        component.subtitles.add(subtitleInfo);
        subtitleNode.getOnMouseClicked().handle(subtitleEvent);
        WaitForAsyncUtils.waitForFxEvents();

        verify(playlistManager).play(media, quality);
        verify(subtitleService).retrieveSubtitles(media);
    }
}