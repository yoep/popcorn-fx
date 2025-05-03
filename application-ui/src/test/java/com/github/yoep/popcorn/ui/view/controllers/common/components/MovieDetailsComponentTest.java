package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class MovieDetailsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private DetailsComponentService service;
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private PlayerManagerService playerService;
    @Mock
    private ISubtitleService subtitleService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private Subtitle.Info subtitleNone;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private MovieDetailsComponent component;

    private final AtomicReference<DetailsComponentListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, DetailsComponentListener.class));
            return null;
        }).when(service).addListener(isA(DetailsComponentListener.class));
        lenient().when(subtitleNone.getLanguage()).thenReturn(Subtitle.Language.NONE);

        component.detailsContent = new GridPane();
        component.detailsDescription = new GridPane();
        component.title = new Label("title");
        component.overview = new Label("overview");
        component.year = new Label("year");
        component.duration = new Label("duration");
        component.genres = new Label("genres");
        component.backgroundImage = new BackgroundImageCover();
        component.ratingStars = new Stars();
        component.magnetLink = new Icon();

        component.detailsDescription.add(new Label(), 0, 0);
        component.detailsDescription.add(new Label(), 1, 0);
        component.detailsDescription.add(new Label(), 2, 0);
        component.detailsDescription.add(new Label(), 3, 0);
    }

    @Test
    void testInitialize() {
        var magnetTooltipText = "FooBar";
        var tooltip = new Tooltip("");
        when(localeText.get(DetailsMessage.MAGNET_LINK)).thenReturn(magnetTooltipText);
        when(viewLoader.load(MovieDetailsComponent.POSTER_COMPONENT_VIEW)).thenReturn(new Pane());
        when(viewLoader.load(MovieDetailsComponent.ACTIONS_COMPONENT_VIEW)).thenReturn(new Pane());
        component.magnetLink.setTooltip(tooltip);

        component.initialize(location, resourceBundle);

        verify(viewLoader).load(MovieDetailsComponent.POSTER_COMPONENT_VIEW);
        verify(viewLoader).load(MovieDetailsComponent.ACTIONS_COMPONENT_VIEW);
    }
}