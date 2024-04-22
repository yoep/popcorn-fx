package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.StringArray;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import javafx.scene.control.ComboBox;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.MalformedURLException;
import java.net.URL;
import java.util.Collections;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopFilterComponentTest {
    @Mock
    private LocaleText localeText;
    @Mock
    private EventPublisher eventPublisher;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    private URL location;
    @Mock
    private ResourceBundle resources;
    @InjectMocks
    private DesktopFilterComponent component;

    private final AtomicReference<Function<CategoryChangedEvent, CategoryChangedEvent>> listener = new AtomicReference<>();

    @BeforeEach
    void setUp() throws MalformedURLException {
        lenient().doAnswer(invocation -> {
            listener.set(invocation.getArgument(1));
            return null;
        }).when(eventPublisher).register(eq(CategoryChangedEvent.class), isA(Function.class));
        component.genreCombo = new ComboBox<>();
        component.sortByCombo = new ComboBox<>();
        location = new URL("http://localhost");
    }

    @Test
    void testCategoryChangedEvent() {
        var displayText = "Lorem ipsum";
        var event = new CategoryChangedEvent(component, Category.FAVORITES);
        var genres = mock(StringArray.class);
        var sortBy = mock(StringArray.class);
        when(fxLib.retrieve_provider_genres(instance, Category.FAVORITES.getProviderName())).thenReturn(genres);
        when(fxLib.retrieve_provider_sort_by(instance, Category.FAVORITES.getProviderName())).thenReturn(sortBy);
        when(genres.values()).thenReturn(Collections.singletonList("lorem"));
        when(sortBy.values()).thenReturn(Collections.singletonList("ipsum"));
        when(localeText.get("genre_lorem")).thenReturn(displayText);
        when(localeText.get("sort-by_ipsum")).thenReturn(displayText);
        component.initialize(location, resources);

        listener.get().apply(event);
        WaitForAsyncUtils.waitForFxEvents();

        verify(eventPublisher, timeout(200)).publish(new GenreChangeEvent(component, new Genre("lorem", displayText)));
        verify(eventPublisher).publish(new SortByChangeEvent(component, new SortBy("ipsum", displayText)));
    }
}