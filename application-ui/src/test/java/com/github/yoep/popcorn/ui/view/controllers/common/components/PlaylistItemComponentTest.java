package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.layout.GridPane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlaylistItemComponentTest {
    @Mock
    private ImageService imageService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private PlaylistItemComponent component;

    @BeforeEach
    void setUp() {
        component = new PlaylistItemComponent(Playlist.Item.newBuilder()
                .setThumb("http://myimage")
                .setTitle("Foo")
                .setCaption("Bar")
                .build(), imageService);

        component.itemPane = new GridPane();
        component.thumbnail = new ImageCover();
        component.title = new Label();
        component.caption = new Label();
    }

    @Test
    void testInitialize() throws TimeoutException {
        var image = new Image(PlaylistItemComponentTest.class.getResourceAsStream("/posterholder.png"));
        when(imageService.load(isA(String.class))).thenReturn(CompletableFuture.completedFuture(image));
        when(imageService.getPosterPlaceholder()).thenReturn(CompletableFuture.completedFuture(image));

        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.thumbnail.getImage() == image);
        assertEquals("Foo", component.title.getText());
        assertEquals("Bar", component.caption.getText());
    }

    @Test
    void testSetActive() {
        component.setActive(true);
        WaitForAsyncUtils.waitForFxEvents();
        assertTrue(component.itemPane.getStyleClass().contains(PlaylistItemComponent.ACTIVE_CLASS), "expected the active style class to be present");

        component.setActive(false);
        WaitForAsyncUtils.waitForFxEvents();
        assertFalse(component.itemPane.getStyleClass().contains(PlaylistItemComponent.ACTIVE_CLASS), "expected the active style class to have been removed");
    }
}