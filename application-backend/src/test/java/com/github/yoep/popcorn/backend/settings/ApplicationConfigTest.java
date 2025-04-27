package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ApplicationConfigTest {
    @Mock
    private LocaleText localeText;
    @Mock
    private FxChannel fxChannel;
    private ApplicationConfig config;

    private final AtomicReference<FxCallback<ApplicationSettingsEvent>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            subscriptionHolder.set((FxCallback<ApplicationSettingsEvent>) invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(ApplicationSettingsEvent.class)), isA(Parser.class), isA(FxCallback.class));
        when(fxChannel.send(isA(ApplicationSettingsRequest.class), isA(Parser.class))).thenAnswer(invocations -> CompletableFuture.completedFuture(ApplicationSettingsResponse.newBuilder()
                .setSettings(ApplicationSettings.newBuilder().build())
                .build()));

        config = new ApplicationConfig(fxChannel, localeText);
    }

    @Test
    void testGetSettings() {
        var request = new AtomicReference<ApplicationSettingsRequest>();
        when(fxChannel.send(isA(ApplicationSettingsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationSettingsRequest.class));
            return CompletableFuture.completedFuture(ApplicationSettingsResponse.newBuilder()
                    .setSettings(ApplicationSettings.newBuilder().build())
                    .build());
        });

        var result = config.getSettings().resultNow();

        verify(fxChannel, times(2)).send(isA(ApplicationSettingsRequest.class), isA(Parser.class));
        assertNotNull(result, "expected to have retrieved settings");
        assertNotNull(request, "expected a request to have been sent");
    }

    @Test
    void testIsTvMode() {
        var request = new AtomicReference<ApplicationArgsRequest>();
        when(fxChannel.send(isA(ApplicationArgsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationArgsRequest.class));
            return CompletableFuture.completedFuture(ApplicationArgsResponse.newBuilder()
                    .setArgs(ApplicationArgs.newBuilder()
                            .setIsTvMode(true)
                            .build())
                    .build());
        });

        var result = config.isTvMode();

        verify(fxChannel).send(isA(ApplicationArgsRequest.class), isA(Parser.class));
        assertTrue(result, "expected result to be true");
        assertNotNull(request, "expected a request to have been sent");
    }

    @Test
    void testIsMaximized() {
        var request = new AtomicReference<ApplicationArgsRequest>();
        when(fxChannel.send(isA(ApplicationArgsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationArgsRequest.class));
            return CompletableFuture.completedFuture(ApplicationArgsResponse.newBuilder()
                    .setArgs(ApplicationArgs.newBuilder()
                            .setIsMaximized(true)
                            .build())
                    .build());
        });

        var result = config.isMaximized();

        verify(fxChannel).send(isA(ApplicationArgsRequest.class), isA(Parser.class));
        assertTrue(result, "expected result to be true");
        assertNotNull(request, "expected a request to have been sent");
    }

    @Test
    void testIsKioskMode() {
        var request = new AtomicReference<ApplicationArgsRequest>();
        when(fxChannel.send(isA(ApplicationArgsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationArgsRequest.class));
            return CompletableFuture.completedFuture(ApplicationArgsResponse.newBuilder()
                    .setArgs(ApplicationArgs.newBuilder()
                            .setIsKioskMode(true)
                            .build())
                    .build());
        });

        var result = config.isKioskMode();

        verify(fxChannel).send(isA(ApplicationArgsRequest.class), isA(Parser.class));
        assertTrue(result, "expected result to be true");
        assertNotNull(request, "expected a request to have been sent");
    }

    @Test
    void testIsMouseDisabled() {
        var request = new AtomicReference<ApplicationArgsRequest>();
        when(fxChannel.send(isA(ApplicationArgsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationArgsRequest.class));
            return CompletableFuture.completedFuture(ApplicationArgsResponse.newBuilder()
                    .setArgs(ApplicationArgs.newBuilder()
                            .setIsMouseDisabled(true)
                            .build())
                    .build());
        });

        var result = config.isMouseDisabled();

        verify(fxChannel).send(isA(ApplicationArgsRequest.class), isA(Parser.class));
        assertTrue(result, "expected result to be true");
        assertNotNull(request, "expected a request to have been sent");
    }

    @Test
    void testUpdate_SubtitleSettings() {
        var settings = ApplicationSettings.SubtitleSettings.newBuilder()
                .setDefaultSubtitle(Subtitle.Language.CROATIAN)
                .build();
        var request = new AtomicReference<UpdateSubtitleSettingsRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdateSubtitleSettingsRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdateSubtitleSettingsRequest.class));

        config.update(settings);

        verify(fxChannel).send(isA(UpdateSubtitleSettingsRequest.class));
        assertEquals(settings, request.get().getSettings());
    }

    @Test
    void testUpdate_TorrentSettings() {
        var settings = ApplicationSettings.TorrentSettings.newBuilder()
                .setCleaningMode(ApplicationSettings.TorrentSettings.CleaningMode.OFF)
                .build();
        var request = new AtomicReference<UpdateTorrentSettingsRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdateTorrentSettingsRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdateTorrentSettingsRequest.class));

        config.update(settings);

        verify(fxChannel).send(isA(UpdateTorrentSettingsRequest.class));
        assertEquals(settings, request.get().getSettings());
    }

    @Test
    void testUpdate_UiSettings() {
        var settings = ApplicationSettings.UISettings.newBuilder()
                .setDefaultLanguage("en")
                .build();
        var request = new AtomicReference<UpdateUISettingsRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdateUISettingsRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdateUISettingsRequest.class));

        config.update(settings);

        verify(fxChannel).send(isA(UpdateUISettingsRequest.class));
        assertEquals(settings, request.get().getSettings());
    }

    @Test
    void testUpdate_ServerSettings() {
        var settings = ApplicationSettings.ServerSettings.newBuilder()
                .setApiServer("http://localhost:98766/api")
                .build();
        var request = new AtomicReference<UpdateServerSettingsRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdateServerSettingsRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdateServerSettingsRequest.class));

        config.update(settings);

        verify(fxChannel).send(isA(UpdateServerSettingsRequest.class));
        assertEquals(settings, request.get().getSettings());
    }

    @Test
    void testUpdate_PlaybackSettings() {
        var settings = ApplicationSettings.PlaybackSettings.newBuilder()
                .setAutoPlayNextEpisodeEnabled(true)
                .build();
        var request = new AtomicReference<UpdatePlaybackSettingsRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdatePlaybackSettingsRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdatePlaybackSettingsRequest.class));

        config.update(settings);

        verify(fxChannel).send(isA(UpdatePlaybackSettingsRequest.class));
        assertEquals(settings, request.get().getSettings());
    }

    @Test
    void testSupportedUiScales() {
        var result = ApplicationConfig.supportedUIScales();

        assertNotNull(result, "expected to have retrieved supported UI scales");
        assertFalse(result.isEmpty(), "expected to have at least retrieved 1 supported UI scale");
    }

    @Test
    void testOnApplicationSettingsEvent_SubtitleSettingsChanged() {
        var listener = mock(ApplicationSettingsEventListener.class);
        var newSettings = ApplicationSettings.SubtitleSettings.newBuilder()
                .setBold(true)
                .setFontFamily(ApplicationSettings.SubtitleSettings.Family.VERDANA)
                .build();
        var event = ApplicationSettingsEvent.newBuilder()
                .setEvent(ApplicationSettingsEvent.Event.SUBTITLE_SETTINGS_CHANGED)
                .setSubtitleSettings(newSettings)
                .build();
        config.addListener(listener);

        subscriptionHolder.get().callback(event);

        verify(listener).onSubtitleSettingsChanged(newSettings);
    }

    @Test
    void testOnApplicationSettingsEvent_TrackingSettingsChanged() {
        var listener = mock(ApplicationSettingsEventListener.class);
        var newSettings = ApplicationSettings.TrackingSettings.newBuilder()
                .setLastSync(ApplicationSettings.TrackingSettings.LastSync.newBuilder().build())
                .build();
        var event = ApplicationSettingsEvent.newBuilder()
                .setEvent(ApplicationSettingsEvent.Event.TRACKING_SETTINGS_CHANGED)
                .setTrackingSettings(newSettings)
                .build();
        config.addListener(listener);

        subscriptionHolder.get().callback(event);

        verify(listener).onTrackingSettingsChanged(newSettings);
    }
}