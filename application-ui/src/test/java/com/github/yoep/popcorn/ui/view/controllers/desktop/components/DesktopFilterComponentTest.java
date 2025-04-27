package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.google.protobuf.Parser;
import javafx.scene.control.ComboBox;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.MalformedURLException;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopFilterComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FxChannel fxChannel;
    @Mock
    private LocaleText localeText;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resources;
    private DesktopFilterComponent component;

    @BeforeEach
    void setUp() throws MalformedURLException {
        component = new DesktopFilterComponent(fxChannel, localeText, eventPublisher);

        component.genreCombo = new ComboBox<>();
        component.sortByCombo = new ComboBox<>();
    }

    @Test
    void testOnCategoryChangedEvent() {
        var expectedGenreText = "FooBar";
        var expectedSortByText = "Lorem";
        lenient().when(localeText.get("genre_foo")).thenReturn(expectedGenreText);
        lenient().when(localeText.get("sort-by_lorem")).thenReturn(expectedSortByText);
        when(fxChannel.send(isA(GetCategoryGenresRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetCategoryGenresResponse.newBuilder()
                .setResult(Response.Result.OK)
                .addGenres(Media.Genre.newBuilder()
                        .setKey("foo")
                        .build())
                .build()));
        when(fxChannel.send(isA(GetCategorySortByRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetCategorySortByResponse.newBuilder()
                .setResult(Response.Result.OK)
                .addSortBy(Media.SortBy.newBuilder()
                        .setKey("lorem")
                        .build())
                .build()));
        component.initialize(location, resources);

        eventPublisher.publishEvent(new CategoryChangedEvent(this, Media.Category.MOVIES));
        WaitForAsyncUtils.waitForFxEvents();

        verify(fxChannel).send(isA(GetCategoryGenresRequest.class), isA(Parser.class));
        verify(fxChannel).send(isA(GetCategorySortByRequest.class), isA(Parser.class));
    }
}