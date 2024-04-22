package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.tracking.TrackingListener;
import com.github.yoep.popcorn.backend.media.tracking.TrackingService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationConfigEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfigEventCallback;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.LastSync;
import com.github.yoep.popcorn.backend.settings.models.TrackingSettings;
import com.github.yoep.popcorn.backend.settings.models.TrackingSyncState;
import com.github.yoep.popcorn.backend.utils.LocaleText;
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
import java.time.LocalDateTime;
import java.util.Optional;
import java.util.ResourceBundle;
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
    private final AtomicReference<ApplicationConfigEventCallback> configListener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            trackingListener.set(invocation.getArgument(0, TrackingListener.class));
            return null;
        }).when(trackingService).addListener(isA(TrackingListener.class));
        doAnswer(invocation -> {
            configListener.set(invocation.getArgument(0, ApplicationConfigEventCallback.class));
            return null;
        }).when(applicationConfig).register(isA(ApplicationConfigEventCallback.class));

        component.statusText = new Label();
        component.authorizeBtn = new Button();
        component.authorizeIcn = new Icon();
        component.syncState = new Label();
        component.syncTime = new Label();
    }

    @Test
    void testOnAuthorizationStateChanged() {
        var expectedText = "disconnect";
        var settings = mock(ApplicationSettings.class);
        var tracking = mock(TrackingSettings.class);
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(settings.getTrackingSettings()).thenReturn(tracking);
        when(localeText.get(SettingsMessage.AUTHORIZE)).thenReturn("connect");
        when(localeText.get(SettingsMessage.DISCONNECT)).thenReturn(expectedText);
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
        var settings = mock(ApplicationSettings.class);
        var tracking = mock(TrackingSettings.class);
        var lastSync = new LastSync();
        var event = new ApplicationConfigEvent.ByValue();
        lastSync.time = 1705739400L;
        lastSync.state = TrackingSyncState.SUCCESS;
        event.tag = ApplicationConfigEvent.Tag.TRACKING_SETTINGS_CHANGED;
        event.union = new ApplicationConfigEvent.ApplicationConfigEventUnion.ByValue();
        event.union.trackingSettingsChanged_body = new ApplicationConfigEvent.TrackingSettingsChanged_Body();
        event.union.trackingSettingsChanged_body.settings = tracking;
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(settings.getTrackingSettings()).thenReturn(tracking);
        when(tracking.getLastSync()).thenReturn(Optional.of(lastSync));
        when(localeText.get(isA(SettingsMessage.class))).thenReturn("Foo");
        when(localeText.get(eq(SettingsMessage.LAST_SYNC_STATE), isA(String.class))).thenReturn(expectedStateMessage);
        when(localeText.get(eq(SettingsMessage.LAST_SYNC_TIME), isA(LocalDateTime.class))).thenReturn(expectedTimeMessage);
        component.initialize(url, resourceBundle);

        var listener = configListener.get();
        listener.callback(event);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(expectedStateMessage, component.syncState.getText());
        assertEquals(expectedTimeMessage, component.syncTime.getText());
        verify(localeText, atLeast(1)).get(SettingsMessage.SYNC_SUCCESS);
        verify(localeText, atLeast(1)).get(SettingsMessage.LAST_SYNC_STATE, "Foo");
    }

    @Test
    void testOnAuthorizationClicked() {
        var event1 = mock(MouseEvent.class);
        var event2 = mock(MouseEvent.class);
        var settings = mock(ApplicationSettings.class);
        var tracking = mock(TrackingSettings.class);
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(settings.getTrackingSettings()).thenReturn(tracking);
        when(trackingService.isAuthorized())
                .thenReturn(true)
                .thenReturn(true)
                .thenReturn(false);
        component.initialize(url, resourceBundle);

        component.onAuthorizeClicked(event1);
        verify(event1).consume();
        verify(trackingService).disconnect();

        component.onAuthorizeClicked(event2);
        verify(event2).consume();
        verify(trackingService).authorize();
    }
}