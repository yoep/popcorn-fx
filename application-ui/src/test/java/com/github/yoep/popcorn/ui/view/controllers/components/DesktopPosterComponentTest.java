package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import javafx.scene.control.Tooltip;
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

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopPosterComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher();
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;
    @Mock
    private LocaleText localeText;
    @InjectMocks
    private DesktopPosterComponent component;

    @BeforeEach
    void setUp() {
        component.posterHolder = new Pane();
        component.watchedIcon = new Icon("watchedIcon");
        component.watchedTooltip = new Tooltip();
        component.favoriteIcon = new Icon("favoriteIcon");
        component.favoriteTooltip = new Tooltip();
    }

    @Test
    void testOnPlayEvent() throws ExecutionException, InterruptedException, TimeoutException {
        var event = mock(PlayMediaEvent.class);
        var media = mock(Media.class);
        var future = new CompletableFuture<Void>();
        when(event.getMedia()).thenReturn(media);
        when(favoriteService.isLiked(media)).thenReturn(true);
        when(watchedService.isWatched(media)).thenReturn(true);
        eventPublisher.register(PlayMediaEvent.class, e -> {
            future.complete(null);
            return e;
        }, EventPublisher.LOWEST_ORDER);
        component.init();

        eventPublisher.publishEvent(event);
        future.get(200, TimeUnit.MILLISECONDS);
        WaitForAsyncUtils.waitForFxEvents();

        assertTrue(component.favoriteIcon.getStyleClass().contains(DesktopPosterComponent.LIKED_STYLE_CLASS));
        assertTrue(component.watchedIcon.getStyleClass().contains(DesktopPosterComponent.WATCHED_STYLE_CLASS));
        verify(localeText).get(DetailsMessage.MARK_AS_NOT_SEEN);
        verify(localeText).get(DetailsMessage.REMOVE_FROM_BOOKMARKS);
    }
}