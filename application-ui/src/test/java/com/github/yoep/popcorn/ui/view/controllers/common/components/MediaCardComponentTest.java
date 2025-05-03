package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.isA;
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

    @Test
    void testInitialize() {
        var title = "lorem";
        var year = "2010";
        var image = new Image(MediaCardComponentTest.class.getResourceAsStream("/posterholder.png"));
        when(media.title()).thenReturn(title);
        when(media.year()).thenReturn(year);
        when(imageService.getPosterPlaceholder(isA(Double.class), isA(Double.class))).thenReturn(CompletableFuture.completedFuture(image));
        when(imageService.loadPoster(media, AbstractCardComponent.POSTER_WIDTH, AbstractCardComponent.POSTER_HEIGHT)).thenReturn(new CompletableFuture<>());
        when(metadataProvider.isWatched(isA(Media.class))).thenReturn(CompletableFuture.completedFuture(true));
        when(metadataProvider.isLiked(isA(Media.class))).thenReturn(CompletableFuture.completedFuture(false));
        var component = createComponent();

        component.initialize(location, resources);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(title, component.title.getText());
        assertEquals(year, component.year.getText());
        assertNotNull(component.poster.getBackground(), "expected the background to have been set");
    }

    private MediaCardComponent createComponent() {
        var component = new MediaCardComponent(media, localeText, imageService, metadataProvider);
        component.title = new Label();
        component.year = new Label();
        component.poster = new Pane();
        component.posterItem = new Pane();
        component.ratingValue = new Label();
        component.ratingStars = new Stars();
        component.favorite = new Icon();
        return component;
    }
}