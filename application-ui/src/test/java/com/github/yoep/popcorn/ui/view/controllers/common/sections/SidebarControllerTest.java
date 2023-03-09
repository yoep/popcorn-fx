package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.view.controls.SearchField;
import javafx.animation.Animation;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Disabled;
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
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SidebarControllerTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private UISettings settings;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    private SearchField searchInput;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SidebarController controller;

    @BeforeEach
    void setUp() {
        lenient().when(applicationConfig.getSettings()).thenReturn(applicationSettings);
        lenient().when(applicationSettings.getUiSettings()).thenReturn(settings);
        searchInput = spy(new SearchField());
        controller.sidebar = new GridPane();
        controller.searchIcon = new Icon("searchIcon");
        controller.searchInput = searchInput;
        controller.movieIcon = new Icon("movieIcon");
        controller.movieText = new Label("movieText");
        controller.serieIcon = new Icon("serieIcon");
        controller.serieText = new Label("serieText");
        controller.favoriteIcon = new Icon("favoriteIcon");
        controller.favoriteText = new Label("favoriteText");
        controller.settingsIcon = new Icon("settingsIcon");
        controller.settingsText = new Label("settingsText");
        controller.infoIcon = new Icon("infoIcon");
        controller.infoText = new Label("infoText");

        controller.sidebar.getColumnConstraints().add(new ColumnConstraints());
        controller.sidebar.getColumnConstraints().add(new ColumnConstraints());
        controller.movieText.setLabelFor(controller.movieIcon);
        controller.serieText.setLabelFor(controller.serieIcon);
        controller.favoriteText.setLabelFor(controller.favoriteIcon);
        controller.settingsText.setLabelFor(controller.settingsIcon);
        controller.infoText.setLabelFor(controller.infoIcon);
        controller.searchIcon.setOnMouseClicked(controller::onSearchClicked);
        controller.searchIcon.setOnKeyPressed(controller::onSearchPressed);
        controller.infoIcon.setOnKeyPressed(controller::onInfoPressed);
    }

    @Test
    void testInitialize_shouldActivePreferredDefaultCategory() {
        when(settings.getStartScreen()).thenReturn(Category.SERIES);
        var expectedEvent = new CategoryChangedEvent(controller, Category.SERIES);

        controller.initialize(url, resourceBundle);

        verify(eventPublisher, timeout(250)).publish(expectedEvent);
        assertEquals(controller.lastKnownSelectedCategory, Category.SERIES);
        assertFalse(controller.movieIcon.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertFalse(controller.movieText.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertFalse(controller.favoriteIcon.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertFalse(controller.favoriteText.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertTrue(controller.serieIcon.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertTrue(controller.serieText.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
    }

    @Test
    void testOnCategoryClicked() {
        var event = mock(MouseEvent.class);
        when(event.getSource()).thenReturn(controller.favoriteIcon);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        when(event.getSource()).thenReturn(controller.favoriteIcon);
        controller.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        controller.onCategoryClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.FAVORITES));
        assertEquals(Category.FAVORITES, controller.lastKnownSelectedCategory);
    }

    @Test
    void testOnCategoryClicked_LabelForIcon() {
        var movieEvent = mock(MouseEvent.class);
        when(movieEvent.getSource()).thenReturn(controller.movieText);
        var serieEvent = mock(MouseEvent.class);
        when(serieEvent.getSource()).thenReturn(controller.serieText);
        var favoriteEvent = mock(MouseEvent.class);
        when(favoriteEvent.getSource()).thenReturn(controller.favoriteText);

        controller.onCategoryClicked(movieEvent);
        verify(movieEvent).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.MOVIES));

        controller.onCategoryClicked(serieEvent);
        verify(serieEvent).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.SERIES));

        controller.onCategoryClicked(favoriteEvent);
        verify(favoriteEvent).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.FAVORITES));
    }

    @Test
    void testOnCategoryPressed() {
        var event = mock(KeyEvent.class);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        when(event.getSource()).thenReturn(controller.favoriteIcon);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        controller.initialize(url, resourceBundle);

        controller.onCategoryPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.FAVORITES));
    }

    @Test
    void testOnHovering() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onHovering(event);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(Animation.Status.RUNNING, controller.slideAnimation.getStatus());
        assertEquals(1, controller.slideAnimation.getToValue());
    }

    @Test
    void testOnHoverStopped() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onHoverStopped(event);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(Animation.Status.RUNNING, controller.slideAnimation.getStatus());
        assertEquals(0, controller.slideAnimation.getToValue());
    }

    @Test
    void testOnSettingsClicked() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onSettingsClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ShowSettingsEvent(controller));
    }

    @Test
    void testOnSettingsPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onSettingsPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ShowSettingsEvent(controller));
    }

    @Test
    void testOnInfoClicked() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onInfoClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ShowAboutEvent(controller));
    }

    @Test
    void testOnInfoPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onInfoPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ShowAboutEvent(controller));
    }

    @Test
    void testOnCloseSettingsEvent_shouldActiveLastKnownCategory() throws ExecutionException, InterruptedException, TimeoutException {
        var event = mock(MouseEvent.class);
        var trigger = new CompletableFuture<Void>();
        when(event.getSource()).thenReturn(controller.serieIcon);
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        eventPublisher.register(ShowSettingsEvent.class, e -> {
            trigger.complete(null);
            return null;
        }, EventPublisher.LOWEST_ORDER);
        controller.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        controller.onCategoryClicked(event);
        controller.onSettingsClicked(mock(MouseEvent.class));
        verify(eventPublisher).publish(new ShowSettingsEvent(controller));

        eventPublisher.publish(new CloseSettingsEvent(this));
        trigger.get(100, TimeUnit.MILLISECONDS);
        WaitForAsyncUtils.waitForFxEvents();
        assertTrue(controller.serieIcon.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
    }

    @Test
    void testOnSearchClicked() {
        var event = mock(MouseEvent.class);

        controller.searchIcon.getOnMouseClicked().handle(event);
        WaitForAsyncUtils.waitForFxEvents();

        verify(event).consume();
        verify(searchInput).requestFocus();
    }

    @Test
    void testOnSearchPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);

        controller.searchIcon.getOnKeyPressed().handle(event);
        WaitForAsyncUtils.waitForFxEvents();

        verify(event).consume();
        verify(searchInput).requestFocus();
    }

    // TODO: Fix the searchfield
    @Test
    @Disabled
    void testOnSearchValueChanged() {
        var value = "lorem";
        when(settings.getStartScreen()).thenReturn(Category.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.searchInput.setText(value);
        WaitForAsyncUtils.waitForFxEvents();

        verify(eventPublisher, timeout(500)).publish(new SearchEvent(this, value));
    }
}