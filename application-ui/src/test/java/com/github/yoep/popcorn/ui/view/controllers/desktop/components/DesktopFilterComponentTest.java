package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
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
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.lenient;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopFilterComponentTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private LocaleText localeText;
    @Mock
    private EventPublisher eventPublisher;
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
    void testInitialize() {
        var expectedGenreText = "FooBar";
        var expectedSortByText = "Lorem";

        component.initialize(location, resources);

        component.genreCombo.getItems().add(Media.Genre.newBuilder()
                .setText(expectedGenreText)
                .build());
        WaitForAsyncUtils.waitForFxEvents(10);
        assertEquals(expectedGenreText, component.genreCombo.getButtonCell().getText());

        component.sortByCombo.getItems().add(Media.SortBy.newBuilder()
                .setText(expectedSortByText)
                .build());
        WaitForAsyncUtils.waitForFxEvents(10);
        assertEquals(expectedSortByText, component.sortByCombo.getButtonCell().getText());
    }
}