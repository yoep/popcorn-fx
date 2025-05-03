package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SearchEvent;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import com.google.protobuf.Parser;
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
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

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
    private FxChannel fxChannel;
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

        eventPublisher.publish(new CategoryChangedEvent(this, Media.Category.MOVIES));
        assertEquals("", component.virtualKeyboard.getText());
        assertEquals("", component.searchValue.getText());
    }

    @Test
    void testOnGenreUpdated() {
        var category = Media.Category.MOVIES;
        var genre = Media.Genre.newBuilder()
                .setKey("lorem")
                .setText("ipsum")
                .build();
        var request = new AtomicReference<GetCategoryGenresRequest>();
        when(fxChannel.send(isA(GetCategoryGenresRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetCategoryGenresRequest.class));
            return CompletableFuture.completedFuture(GetCategoryGenresResponse.newBuilder()
                    .setResult(Response.Result.OK)
                    .addGenres(genre)
                    .build());
        });
        when(fxChannel.send(isA(GetCategorySortByRequest.class), isA(Parser.class))).thenAnswer(invocations -> CompletableFuture.completedFuture(GetCategorySortByResponse.newBuilder()
                .setResult(Response.Result.OK)
                .addSortBy(Media.SortBy.newBuilder()
                        .setKey("lorem")
                        .build())
                .build()));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        eventPublisher.publishEvent(new CategoryChangedEvent(this, category));
        WaitForAsyncUtils.waitForFxEvents();
        assertEquals(category, request.get().getCategory());
        assertTrue(component.genres.getItems().stream().anyMatch(e -> e.getKey().equals("lorem")));

        component.genres.setSelectedItem(genre);
        WaitForAsyncUtils.waitForFxEvents();

        verify(eventPublisher).publish(isA(GenreChangeEvent.class));
        verify(localeText).get("genre_lorem");
    }
}