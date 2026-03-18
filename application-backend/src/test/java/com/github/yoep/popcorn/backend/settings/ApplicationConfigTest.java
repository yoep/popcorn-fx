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

    private final AtomicReference<FxCallback<ApplicationSettingsEvent>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocations -> {
            subscriptionHolder.set((FxCallback<ApplicationSettingsEvent>) invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(ApplicationSettingsEvent.class)), isA(Parser.class), isA(FxCallback.class));
    }

    @Test
    void testInit_shouldSetUiScaleIndex() {
        var index = 5;
        var scale = ApplicationConfig.supportedUIScales().get(index);
        when(fxChannel.send(isA(ApplicationSettingsRequest.class), isA(Parser.class)))
                .thenReturn(CompletableFuture.completedFuture(ApplicationSettingsResponse.newBuilder()
                        .setSettings(ApplicationSettings.newBuilder()
                                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                                        .setScale(scale)
                                        .build())
                                .build())
                        .build()));
        var config = new ApplicationConfig(fxChannel, localeText);

        var result = config.uiScaleIndex;

        assertEquals(index, result, "expected the ui scale index to have been set");
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
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

        config.update(settings);

        verify(fxChannel).send(isA(UpdateUISettingsRequest.class));
        assertEquals(settings, request.get().getSettings());
    }

    @Test
    void testUpdate_ServerSettings() {
        var settings = ApplicationSettings.ServerSettings.newBuilder()
                .addMovieApiServers("http://localhost:98766/api")
                .build();
        var request = new AtomicReference<UpdateServerSettingsRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdateServerSettingsRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdateServerSettingsRequest.class));
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

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
        mockDefaultApplicationSettings();
        var config = new ApplicationConfig(fxChannel, localeText);

        config.addListener(listener);
        subscriptionHolder.get().callback(event);

        verify(listener).onTrackingSettingsChanged(newSettings);
    }

    private void mockDefaultApplicationSettings() {
        when(fxChannel.send(isA(ApplicationSettingsRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(ApplicationSettingsResponse.newBuilder()
                .setSettings(ApplicationSettings.newBuilder().build())
                .build()));
    }
}