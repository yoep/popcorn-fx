package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class MediaCardComponentTest {
    @Mock
    private Media media;
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private OverlayItemMetadataProvider metadataProvider;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resources;
    @InjectMocks
    private MediaCardComponent component;

    @BeforeEach
    void setUp() {
        component.title = new Label();
        component.year = new Label();
        component.poster = new Pane();
        component.posterItem = new Pane();
        component.ratingValue = new Label();
        component.ratingStars = new Stars();
        component.favorite = new Icon();
    }

    @Test
    void testInitialize() {
        var title = "lorem";
        var year = "2010";
        when(media.getTitle()).thenReturn(title);
        when(media.getYear()).thenReturn(year);
        when(imageService.loadPoster(media, AbstractCardComponent.POSTER_WIDTH, AbstractCardComponent.POSTER_HEIGHT)).thenReturn(new CompletableFuture<>());

        component.initialize(location, resources);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(title, component.title.getText());
        assertEquals(year, component.year.getText());
    }
}