package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.listeners.SubtitleListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.Node;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PopcornPlayerSectionServiceTest {
    @Mock
    private PopcornPlayer player;
    @Mock
    private ScreenService screenService;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private SubtitleManagerService subtitleManagerService;
    @Mock
    private VideoService videoService;
    @Mock
    private PopcornPlayerSectionListener listener;
    @Mock
    private ApplicationSettings settings;

    private final ObjectProperty<VideoPlayback> videoPlayerProperty = new SimpleObjectProperty<>();
    private final IntegerProperty subtitleSizeProperty = new SimpleIntegerProperty();
    private final AtomicReference<PlayerListener> listenerHolder = new AtomicReference<>();
    private final AtomicReference<SubtitleListener> subtitleListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        lenient().when(videoService.videoPlayerProperty()).thenReturn(videoPlayerProperty);
        lenient().when(applicationConfig.getSettings()).thenReturn(settings);
        lenient().when(subtitleManagerService.subtitleSizeProperty()).thenReturn(subtitleSizeProperty);
        lenient().doAnswer(invocation -> {
            subtitleListenerHolder.set(invocation.getArgument(0, SubtitleListener.class));
            return null;
        }).when(subtitleManagerService).registerListener(isA(SubtitleListener.class));
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPlaying_shouldPausePlayer() {
        when(player.getState()).thenReturn(PlayerState.PLAYING);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.togglePlayerPlaybackState();

        verify(player).pause();
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPaused_shouldResumePlayer() {
        when(player.getState()).thenReturn(PlayerState.PAUSED);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.togglePlayerPlaybackState();

        verify(player).resume();
    }

    @Test
    void testToggleFullscreen_whenInvoked_shouldToggleFullscreen() {
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.toggleFullscreen();

        verify(screenService).toggleFullscreen();
    }

    @Test
    void testVideoTimeOffset_whenOffsetIsPositive_shouldSeekInTheFuture() {
        var offset = 2000;
        var currentTime = 10000L;
        when(player.getTime()).thenReturn(currentTime);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.videoTimeOffset(offset);

        verify(player).seek(currentTime + offset);
    }

    @Test
    void testVideoTimeOffset_whenOffsetIsNegative_shouldSeekInThePast() {
        var offset = -1000;
        var currentTime = 20000L;
        when(player.getTime()).thenReturn(currentTime);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.videoTimeOffset(offset);

        verify(player).seek(currentTime + offset);
    }

    @Test
    void testIsNativeSubtitlePlaybackSupported_whenVideoPlayerIsActive_shouldReturnTheNativeSubtitleState() {
        var videoPlayer = mock(VideoPlayback.class);
        var supportNativeSubtitle = true;
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(supportNativeSubtitle);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        var result = service.isNativeSubtitlePlaybackSupported();

        assertEquals(supportNativeSubtitle, result);
    }

    @Test
    void testIsNativeSubtitlePlaybackSupported_whenThereIsNoVideoPlayerActive_shouldReturnFalse() {
        when(videoService.getVideoPlayer()).thenReturn(Optional.empty());
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        var result = service.isNativeSubtitlePlaybackSupported();

        assertFalse(result);
    }

    @Test
    void testPlayerListener_whenPlayerTimeChanged_shouldInvokedListeners() {
        var value = 10200;
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        listenerHolder.get().onTimeChanged(value);

        verify(listener).onPlayerTimeChanged(value);
    }

    @Test
    void testPlayerListener_whenPlayerStateChanged_shouldInvokedListeners() {
        var value = PlayerState.PLAYING;
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        listenerHolder.get().onStateChanged(value);

        verify(listener).onPlayerStateChanged(value);
    }

    @Test
    void testVideoPlayerListener_whenVideoPlayerIsChanged_shouldInvokeListeners() {
        var videoPlayer = mock(VideoPlayback.class);
        var videoView = mock(Node.class);
        when(videoPlayer.getVideoSurface()).thenReturn(videoView);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        videoPlayerProperty.set(videoPlayer);

        verify(listener).onVideoViewChanged(videoView);
    }

    @Test
    void testProvideSubtitleValues_whenInvoked_shouldSetSubtitleFontFamily() {
        var value = SubtitleFamily.ARIAL;
        var subtitle = mock(SubtitleSettings.ByValue.class);
        when(subtitle.getFontFamily()).thenReturn(value);
        when(settings.getSubtitleSettings()).thenReturn(subtitle);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.provideSubtitleValues();

        verify(listener).onSubtitleFamilyChanged(value.getFamily());
    }

    @Test
    void testProvideSubtitleValues_whenInvoked_shouldSetSubtitleFontWeight() {
        var subtitleSettings = new SubtitleSettings.ByValue();
        subtitleSettings.bold = (byte) 1;
        subtitleSettings.fontFamily = SubtitleFamily.ARIAL;
        when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.provideSubtitleValues();

        verify(listener).onSubtitleFontWeightChanged(true);
    }

    @Test
    void testProvideSubtitleValues_whenInvoked_shouldSetSubtitleSize() {
        var fontSize = 22;
        var subtitle = mock(SubtitleSettings.ByValue.class);
        when(subtitle.getFontSize()).thenReturn(fontSize);
        when(settings.getSubtitleSettings()).thenReturn(subtitle);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.provideSubtitleValues();

        verify(listener).onSubtitleSizeChanged(fontSize);
    }

    @Test
    void testProvideSubtitleValues_whenInvoked_shouldSetSubtitleDecoration() {
        var value = DecorationType.OUTLINE;
        var subtitleSettings = new SubtitleSettings.ByValue();
        subtitleSettings.decoration = value;
        subtitleSettings.fontFamily = SubtitleFamily.ARIAL;
        when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.provideSubtitleValues();

        verify(listener).onSubtitleDecorationChanged(value);
    }

    @Test
    void testSubtitleSettingsListener_whenFontFamilyIsChanged_shouldInvokeListeners() {
        var value = SubtitleFamily.ARIAL.getFamily();
        var subtitleSettings = mock(SubtitleSettings.class);
        var settingsListener = new AtomicReference<ApplicationConfigEventCallback>();
        var event = new ApplicationConfigEvent.ByValue();
        event.tag = ApplicationConfigEvent.Tag.SUBTITLE_SETTINGS_CHANGED;
        event.union = new ApplicationConfigEvent.ApplicationConfigEventUnion.ByValue();
        event.union.subtitleSettingsChanged_body = new ApplicationConfigEvent.SubtitleSettingsChanged_Body();
        event.union.subtitleSettingsChanged_body.settings = subtitleSettings;
        doAnswer(invocation -> {
            settingsListener.set(invocation.getArgument(0, ApplicationConfigEventCallback.class));
            return null;
        }).when(applicationConfig).register(isA(ApplicationConfigEventCallback.class));
        when(subtitleSettings.getFontFamily()).thenReturn(SubtitleFamily.ARIAL);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        settingsListener.get().callback(event);

        verify(listener).onSubtitleFamilyChanged(value);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsChanged_shouldInvokedListeners() {
        var subtitle = mock(Subtitle.class);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        var listener = subtitleListenerHolder.get();
        listener.onSubtitleChanged(subtitle);

        verify(this.listener).onSubtitleChanged(subtitle);
    }

    @Test
    void testSubtitleListener_whenSubtitleIsDisabled_shouldInvokedListeners() {
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        var listener = subtitleListenerHolder.get();
        listener.onSubtitleDisabled();

        verify(this.listener).onSubtitleDisabled();
    }

    @Test
    void testSubtitleListener_whenSubtitleSizeIsChanged_shouldInvokedListeners() {
        var subtitleSize = 28;
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        subtitleSizeProperty.set(subtitleSize);

        verify(listener).onSubtitleSizeChanged(subtitleSize);
    }

    @Test
    void testOnVolumeScroll_whenNewVolumeIsAbove100_shouldUpdateVolumeTo100() {
        when(player.getVolume()).thenReturn(95);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.onVolumeScroll(15);

        verify(player).volume(100);
    }

    @Test
    void testOnVolumeScroll_whenNewVolumeIsBelowZero_shouldUpdateVolumeToZero() {
        when(player.getVolume()).thenReturn(10);
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        service.onVolumeScroll(-20);

        verify(player).volume(0);
    }

    @Test
    void testListener_whenPlayerVolumeIsChanged_shouldInvokedListeners() {
        var volume = 75;
        var service = new PopcornPlayerSectionService(player, screenService, applicationConfig, subtitleManagerService, videoService);
        service.addListener(listener);

        listenerHolder.get().onVolumeChanged(volume);

        verify(listener).onVolumeChanged(volume);
    }
}