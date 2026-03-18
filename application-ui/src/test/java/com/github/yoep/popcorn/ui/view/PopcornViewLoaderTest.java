package com.github.yoep.popcorn.ui.view;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationSettingsEventListener;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.IoC;
import com.github.yoep.popcorn.ui.view.controllers.common.components.NotificationComponent;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.UpdateSectionController;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornViewLoaderTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private IoC instance;
    @Mock
    private ViewManager viewManager;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig applicationConfig;

    @Test
    void testInit_shouldLoadUiScale() {
        var scaleFactor = 1.75f;
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                        .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                                .setFactor(scaleFactor)
                                .build())
                        .build())
                .build()));

        var loader = new PopcornViewLoader(instance, applicationConfig, viewManager, localeText);

        assertEquals(scaleFactor, loader.scale);
    }

    @Test
    void testOnUiScaleChanged() {
        var newScale = 2.0f;
        var holder = new AtomicReference<ApplicationSettingsEventListener>();
        doAnswer(invocation -> {
            holder.set(invocation.getArgument(0));
            return null;
        }).when(applicationConfig).addListener(isA(ApplicationSettingsEventListener.class));
        when(applicationConfig.getSettings()).thenReturn(new CompletableFuture<>());
        var loader = new PopcornViewLoader(instance, applicationConfig, viewManager, localeText);

        var listener = holder.get();
        listener.onUiSettingsChanged(ApplicationSettings.UISettings.newBuilder()
                .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                        .setFactor(newScale)
                        .build())
                .build());

        assertEquals(newScale, loader.scale);
    }

    @Test
    void testLoad() {
        var updateService = mock(UpdateService.class);
        var imageService = mock(ImageService.class);
        var controller = new UpdateSectionController(updateService, imageService, localeText, eventPublisher);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(updateService.getState()).thenReturn(new CompletableFuture<>());
        when(instance.getInstance(UpdateSectionController.class)).thenReturn(controller);
        when(applicationConfig.getSettings()).thenReturn(new CompletableFuture<>());
        var loader = new PopcornViewLoader(instance, applicationConfig, viewManager, localeText);

        var result = loader.load("common/sections/update.section.fxml");

        assertNotNull(result, "expected the view to have been loaded");
    }

    @Test
    void testLoadController() {
        when(applicationConfig.getSettings()).thenReturn(new CompletableFuture<>());
        var controller = new NotificationComponent(new InfoNotificationEvent(this, "lorem ipsum dolor"));
        var loader = new PopcornViewLoader(instance, applicationConfig, viewManager, localeText);

        var result = loader.load("common/components/notification.component.fxml", controller);

        assertNotNull(result, "expected the view to have been loaded");
    }
}