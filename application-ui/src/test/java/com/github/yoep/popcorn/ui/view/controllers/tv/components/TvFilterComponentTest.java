package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxStringArray;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SearchEvent;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import javafx.scene.control.Label;
import javafx.scene.layout.VBox;
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
import java.util.ResourceBundle;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvFilterComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private TvFilterComponent component;

    @BeforeEach
    void setUp() {
        component.filter = new VBox();
        component.searchValue = new Label();
        component.virtualKeyboard = new VirtualKeyboard();
        component.genres = new AxisItemSelection<>();
    }

    @Test
    void testOnVirtualKeyboardChanged() {
        var value = "lorem";
        component.initialize(url, resourceBundle);

        component.virtualKeyboard.setText(value);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(value, component.searchValue.getText());
        verify(eventPublisher, timeout(3200)).publish(new SearchEvent(component, value));
    }

    @Test
    void testOnCategoryChanged() {
        component.initialize(url, resourceBundle);
        component.virtualKeyboard.setText("lorem");

        eventPublisher.publish(new CategoryChangedEvent(this, Category.MOVIES));
        assertEquals("", component.virtualKeyboard.getText());
        assertEquals("", component.searchValue.getText());
    }

    @Test
    void testOnCategoryChanged_shouldUpdateGenres() {
        var category = Category.MOVIES;
        var genreValues = mock(FxStringArray.class);
        component.initialize(url, resourceBundle);
        when(genreValues.values()).thenReturn(Collections.singletonList("lorem"));
        when(fxLib.retrieve_provider_genres(instance, category.getProviderName())).thenReturn(genreValues);

        eventPublisher.publish(new CategoryChangedEvent(this, category));
        WaitForAsyncUtils.waitForFxEvents();

        assertTrue(component.genres.getItems().stream().anyMatch(e -> e.getKey().equals("lorem")));
        verify(localeText).get("genre_lorem");
    }

    @Test
    void testOnGenreUpdated() {
        var genre = new Genre("lorem", "ipsum");
        component.initialize(url, resourceBundle);

        component.genres.setSelectedItem(genre);

        verify(eventPublisher).publish(new GenreChangeEvent(component, genre));
    }
}