package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.media.tracking.TrackingListener;
import com.github.yoep.popcorn.backend.media.tracking.TrackingService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationSettingsEventListener;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.backend.utils.Message;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
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
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsTraktComponentTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private TrackingService trackingService;
    @Mock
    private LocaleText localeText;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SettingsTraktComponent component;

    private final AtomicReference<TrackingListener> trackingListener = new AtomicReference<>();
    private final AtomicReference<ApplicationSettingsEventListener> settingsListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            trackingListener.set(invocation.getArgument(0, TrackingListener.class));
            return null;
        }).when(trackingService).addListener(isA(TrackingListener.class));
        doAnswer(invocation -> {
            settingsListenerHolder.set(invocation.getArgument(0, ApplicationSettingsEventListener.class));
            return null;
        }).when(applicationConfig).addListener(isA(ApplicationSettingsEventListener.class));
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setTrackingSettings(ApplicationSettings.TrackingSettings.newBuilder().build())
                .build()));

        component.statusText = new Label();
        component.authorizeBtn = new Button();
        component.authorizeIcn = new Icon();
        component.syncState = new Label();
        component.syncTime = new Label();
    }

    @Test
    void testOnAuthorizationStateChanged() {
        var expectedText = "disconnect";
        when(localeText.get(isA(Message.class))).thenReturn("PLACEHOLDER");
        when(localeText.get(isA(Message.class), any())).thenReturn("PLACEHOLDER");
        when(localeText.get(SettingsMessage.DISCONNECT)).thenReturn(expectedText);
        when(trackingService.isAuthorized()).thenReturn(CompletableFuture.completedFuture(false));
        component.initialize(url, resourceBundle);

        var listener = trackingListener.get();
        listener.onAuthorizationChanged(true);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(expectedText, component.authorizeBtn.getText());
        assertEquals(Icon.CHAIN_BROKEN_UNICODE, component.authorizeIcn.getText());
    }

    @Test
    void testTrackingSettingsChanged() {
        var expectedStateMessage = "expected state message";
        var expectedTimeMessage = "expected time message";
        var settings = ApplicationSettings.newBuilder()
                .setTrackingSettings(ApplicationSettings.TrackingSettings.newBuilder()
                        .setLastSync(ApplicationSettings.TrackingSettings.LastSync.newBuilder()
                                .setLastSyncedMillis(1705739400L)
                                .setState(ApplicationSettings.TrackingSettings.LastSync.State.SUCCESS)
                                .build())
                        .build())
                .build();
        when(localeText.get(isA(SettingsMessage.class))).thenReturn("Foo");
        when(localeText.get(eq(SettingsMessage.LAST_SYNC_STATE), isA(String.class))).thenReturn(expectedStateMessage);
        when(localeText.get(eq(SettingsMessage.LAST_SYNC_TIME), isA(String.class))).thenReturn(expectedTimeMessage);
        when(trackingService.isAuthorized()).thenReturn(CompletableFuture.completedFuture(false));
        component.initialize(url, resourceBundle);

        var listener = settingsListenerHolder.get();
        listener.onTrackingSettingsChanged(settings.getTrackingSettings());
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(expectedStateMessage, component.syncState.getText());
        assertEquals(expectedTimeMessage, component.syncTime.getText());
        verify(localeText, atLeast(1)).get(SettingsMessage.SYNC_SUCCESS);
        verify(localeText, atLeast(1)).get(eq(SettingsMessage.LAST_SYNC_STATE), isA(String.class));
    }

    @Test
    void testOnAuthorizationClicked() {
        var event1 = mock(MouseEvent.class);
        var event2 = mock(MouseEvent.class);
        when(trackingService.isAuthorized())
                .thenReturn(CompletableFuture.completedFuture(true))
                .thenReturn(CompletableFuture.completedFuture(true))
                .thenReturn(CompletableFuture.completedFuture(false));
        component.initialize(url, resourceBundle);

        component.onAuthorizeClicked(event1);
        verify(event1).consume();
        verify(trackingService).disconnect();

        component.onAuthorizeClicked(event2);
        verify(event2).consume();
        verify(trackingService).authorize();
    }
}