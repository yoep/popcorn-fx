package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.invocation.InvocationOnMock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
public class SettingsPlaybackComponentTest {
    @Spy
    private EventPublisher activityManager = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig applicationConfig;
    @InjectMocks
    private SettingsPlaybackComponent settingsPlaybackComponent;

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setPlaybackSettings(ApplicationSettings.PlaybackSettings.newBuilder().build())
                .build()));
    }

    @Test
    void testShowNotification_whenActivityIsInvokedInvoked_shouldCallLocaleTextWithSettingsSaved() {
        doAnswer(this::invokeSuccessNotification).when(activityManager).publishEvent(isA(SuccessNotificationEvent.class));

        settingsPlaybackComponent.onQualityChanged(ApplicationSettings.PlaybackSettings.Quality.P720);

        verify(localeText).get(SettingsMessage.SETTINGS_SAVED);
    }

    @Test
    void testOnQualityChanged_whenInvoked_shouldShowNotification() {
        settingsPlaybackComponent.onQualityChanged(ApplicationSettings.PlaybackSettings.Quality.P1080);

        verify(activityManager).publishEvent(isA(SuccessNotificationEvent.class));
    }

    @Test
    void testOnQualityChanged_whenInvoked_shouldUpdateQualitySetting() {
        var quality = ApplicationSettings.PlaybackSettings.Quality.P1080;
        var settings = new AtomicReference<ApplicationSettings.PlaybackSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.PlaybackSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.PlaybackSettings.class));

        settingsPlaybackComponent.onQualityChanged(quality);

        verify(applicationConfig).update(isA(ApplicationSettings.PlaybackSettings.class));
        assertEquals(quality, settings.get().getQuality());
    }

    @Test
    void testOnFullscreenChanged_whenInvoked_shouldShowNotification() {
        settingsPlaybackComponent.onFullscreenChanged(false);

        verify(activityManager).publishEvent(isA(SuccessNotificationEvent.class));
    }

    @Test
    void testOnFullscreenChanged_whenInvoked_shouldUpdateTheFullscreenSetting() {
        var settings = new AtomicReference<ApplicationSettings.PlaybackSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.PlaybackSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.PlaybackSettings.class));

        settingsPlaybackComponent.onFullscreenChanged(true);

        verify(applicationConfig).update(isA(ApplicationSettings.PlaybackSettings.class));
        assertTrue(settings.get().getFullscreen(), "expected fullscreen to have been set to true");
    }

    @Test
    void testOnAutoPlayNextEpisodeChanged_whenInvoked_shouldShowNotification() {
        settingsPlaybackComponent.onAutoPlayNextEpisodeChanged(true);

        verify(activityManager).publishEvent(isA(SuccessNotificationEvent.class));
    }

    @Test
    void testOnAutoPlayNextEpisodeChanged_whenInvoked_shouldUpdateTheFullscreenSetting() {
        var settings = new AtomicReference<ApplicationSettings.PlaybackSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.PlaybackSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.PlaybackSettings.class));

        settingsPlaybackComponent.onAutoPlayNextEpisodeChanged(false);

        verify(applicationConfig).update(isA(ApplicationSettings.PlaybackSettings.class));
        assertFalse(settings.get().getAutoPlayNextEpisodeEnabled(), "expected auto play next episode to have been set to false");
    }

    private Void invokeSuccessNotification(InvocationOnMock invocation) {
        var activity = (SuccessNotificationEvent) invocation.getArgument(0);
        activity.getText();
        return null;
    }
}
