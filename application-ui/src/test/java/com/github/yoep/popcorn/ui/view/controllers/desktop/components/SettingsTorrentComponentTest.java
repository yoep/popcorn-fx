package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import javafx.scene.control.ComboBox;
import javafx.scene.control.TextField;
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
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsTorrentComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SettingsTorrentComponent component;

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setTorrentSettings(ApplicationSettings.TorrentSettings.newBuilder()
                        .setCleaningMode(ApplicationSettings.TorrentSettings.CleaningMode.OFF)
                        .setDownloadRateLimit(0)
                        .setUploadRateLimit(0)
                        .setConnectionsLimit(0)
                        .build())
                .build()));

        component.cacheDirectory = new TextField();
        component.connectionLimit = new DelayedTextField();
        component.downloadLimit = new DelayedTextField();
        component.uploadLimit = new DelayedTextField();
        component.cleaningMode = new ComboBox<>();
    }

    @Test
    void testOnCleaningModeChanged() {
        var settings = new AtomicReference<ApplicationSettings.TorrentSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.TorrentSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.TorrentSettings.class));
        var expectedCleaningMode = ApplicationSettings.TorrentSettings.CleaningMode.WATCHED;
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.cleaningMode.getSelectionModel().select(expectedCleaningMode);
        WaitForAsyncUtils.waitForFxEvents();

        verify(applicationConfig).update(isA(ApplicationSettings.TorrentSettings.class));
        assertEquals(expectedCleaningMode, settings.get().getCleaningMode());
    }
}