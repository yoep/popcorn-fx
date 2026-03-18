package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaException;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.Collections;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class ListSectionControllerTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ProviderService<Media> providerService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private ListSectionController controller;

    @BeforeEach
    void setUp() {
        controller = new ListSectionController(Collections.singletonList(providerService), favoriteService, watchedService, viewLoader, localeText, eventPublisher, imageService, applicationConfig);

        controller.listSection = new AnchorPane();
        controller.listSection.getChildren().add(new Pane());

        controller.backgroundImage = new BackgroundImageCover();
        controller.retryBtn = spy(new Button());
        controller.scrollPane = spy(new InfiniteScrollPane<>());
        controller.failedPane = new Pane();
        controller.failedText = new Label();
        controller.overlay = new Pane();
    }

    @Test
    void testOnMediaLoadingFailed_shouldFocusRetryButton() {
        var category = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Category.MOVIES;
        var genre = createGenre("action", "Action");
        var sortBy = createSortBy("trending", "Trending");
        when(providerService.getPage(category, genre, sortBy, 1)).thenReturn(CompletableFuture.failedFuture(new MediaException(MediaException.ErrorType.RETRIEVAL, "500 internal server error")));
        when(viewLoader.load(ListSectionController.FILTER_COMPONENT)).thenReturn(new Pane());
        mockBackgroundImageLoading();
        controller.initialize(url, resourceBundle);

        // trigger the events to start loading media items
        eventPublisher.publish(new CategoryChangedEvent(this, category));
        eventPublisher.publish(new GenreChangeEvent(this, genre));
        eventPublisher.publish(new SortByChangeEvent(this, sortBy));
        WaitForAsyncUtils.waitForFxEvents();

        verify(controller.retryBtn).requestFocus();
        assertTrue(controller.failedPane.isVisible(), "expected the failed pane to be visible");
    }

    @Test
    void testOnRetryBtnClicked() {
        var event = mock(MouseEvent.class);
        when(event.getButton()).thenReturn(MouseButton.PRIMARY);
        when(viewLoader.load(ListSectionController.FILTER_COMPONENT)).thenReturn(new Pane());
        mockProviderService();
        mockBackgroundImageLoading();
        controller.category = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Category.MOVIES;
        controller.genre = createGenre("lorem", "Lorem");
        controller.sortBy = createSortBy("ipsum", "Ipsum");
        controller.initialize(url, resourceBundle);

        controller.onRetryBtnClicked(event);
        WaitForAsyncUtils.waitForFxEvents();

        verify(event).consume();
        verify(controller.scrollPane).reset();
        verify(controller.scrollPane).loadNewPage();
    }

    @Test
    void testOnRetryBtnPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(viewLoader.load(ListSectionController.FILTER_COMPONENT)).thenReturn(new Pane());
        mockProviderService();
        mockBackgroundImageLoading();
        controller.category = com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Category.MOVIES;
        controller.genre = createGenre("foo", "Foo");
        controller.sortBy = createSortBy("bar", "Bar");
        controller.initialize(url, resourceBundle);

        controller.onRetryBtnPressed(event);
        WaitForAsyncUtils.waitForFxEvents();

        verify(event).consume();
        verify(controller.scrollPane).reset();
        verify(controller.scrollPane).loadNewPage();
    }

    private void mockBackgroundImageLoading() {
        when(imageService.loadResource(isA(String.class)))
                .thenReturn(CompletableFuture.completedFuture(new Image(ListSectionControllerTest.class.getResourceAsStream("/posterholder.png"))));
    }

    private void mockProviderService() {
        when(providerService.getPage(
                isA(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Category.class),
                isA(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Genre.class),
                isA(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.SortBy.class),
                isA(Integer.class)
        )).thenReturn(new CompletableFuture<>());
    }

    private com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Genre createGenre(String key, String text) {
        return com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Genre.newBuilder()
                .setKey(key)
                .setText(text)
                .build();
    }

    private com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.SortBy createSortBy(String key, String text) {
        return com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.SortBy.newBuilder()
                .setKey(key)
                .setText(text)
                .build();
    }
}