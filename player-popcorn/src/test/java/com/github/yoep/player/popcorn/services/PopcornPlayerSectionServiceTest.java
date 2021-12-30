package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.Node;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
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
    private SettingsService settingsService;
    @Mock
    private SubtitleManagerService subtitleManagerService;
    @Mock
    private VideoService videoService;
    @Mock
    private PopcornPlayerSectionListener listener;
    @Mock
    private ApplicationSettings settings;
    @InjectMocks
    private PopcornPlayerSectionService service;

    private final ObjectProperty<VideoPlayer> videoPlayerProperty = new SimpleObjectProperty<>();
    private final IntegerProperty subtitleSizeProperty = new SimpleIntegerProperty();
    private final ObjectProperty<Subtitle> activeSubtitleProperty =new SimpleObjectProperty<>();
    private final AtomicReference<PlayerListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, PlayerListener.class));
            return null;
        }).when(player).addListener(isA(PlayerListener.class));
        lenient().when(videoService.videoPlayerProperty()).thenReturn(videoPlayerProperty);
        lenient().when(settingsService.getSettings()).thenReturn(settings);
        lenient().when(subtitleManagerService.subtitleSizeProperty()).thenReturn(subtitleSizeProperty);
        lenient().when(subtitleManagerService.activeSubtitleProperty()).thenReturn(activeSubtitleProperty);

        service.addListener(listener);
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPlaying_shouldPausePlayer() {
        when(player.getState()).thenReturn(PlayerState.PLAYING);

        service.togglePlayerPlaybackState();

        verify(player).pause();
    }

    @Test
    void testTogglePlaybackState_whenPlayerIsPaused_shouldResumePlayer() {
        when(player.getState()).thenReturn(PlayerState.PAUSED);

        service.togglePlayerPlaybackState();

        verify(player).resume();
    }

    @Test
    void testToggleFullscreen_whenInvoked_shouldToggleFullscreen() {
        service.toggleFullscreen();

        verify(screenService).toggleFullscreen();
    }

    @Test
    void testVideoTimeOffset_whenOffsetIsPositive_shouldSeekInTheFuture() {
        var offset = 2000;
        var currentTime = 10000L;
        when(player.getTime()).thenReturn(currentTime);

        service.videoTimeOffset(offset);

        verify(player).seek(currentTime + offset);
    }

    @Test
    void testVideoTimeOffset_whenOffsetIsNegative_shouldSeekInThePast() {
        var offset = -1000;
        var currentTime = 20000L;
        when(player.getTime()).thenReturn(currentTime);

        service.videoTimeOffset(offset);

        verify(player).seek(currentTime + offset);
    }

    @Test
    void testIsNativeSubtitlePlaybackSupported_whenVideoPlayerIsActive_shouldReturnTheNativeSubtitleState() {
        var videoPlayer = mock(VideoPlayer.class);
        var supportNativeSubtitle = true;
        when(videoService.getVideoPlayer()).thenReturn(Optional.of(videoPlayer));
        when(videoPlayer.supportsNativeSubtitleFile()).thenReturn(supportNativeSubtitle);

        var result = service.isNativeSubtitlePlaybackSupported();

        assertEquals(supportNativeSubtitle, result);
    }

    @Test
    void testIsNativeSubtitlePlaybackSupported_whenThereIsNoVideoPlayerActive_shouldReturnFalse() {
        when(videoService.getVideoPlayer()).thenReturn(Optional.empty());

        var result = service.isNativeSubtitlePlaybackSupported();

        assertFalse(result);
    }

    @Test
    void testPlayerListener_whenPlayerTimeChanged_shouldInvokedListeners() {
        var value = 10200;
        when(settings.getSubtitleSettings()).thenReturn(SubtitleSettings.builder().build());
        service.init();

        listenerHolder.get().onTimeChanged(value);

        verify(listener).onPlayerTimeChanged(value);
    }

    @Test
    void testPlayerListener_whenPlayerStateChanged_shouldInvokedListeners() {
        var value = PlayerState.PLAYING;
        when(settings.getSubtitleSettings()).thenReturn(SubtitleSettings.builder().build());
        service.init();

        listenerHolder.get().onStateChanged(value);

        verify(listener).onPlayerStateChanged(value);
    }

    @Test
    void testVideoPlayerListener_whenVideoPlayerIsChanged_shouldInvokeListeners() {
        var videoPlayer = mock(VideoPlayer.class);
        var videoView = mock(Node.class);
        when(videoPlayer.getVideoSurface()).thenReturn(videoView);
        when(settings.getSubtitleSettings()).thenReturn(SubtitleSettings.builder().build());
        service.init();

        videoPlayerProperty.set(videoPlayer);

        verify(listener).onVideoViewChanged(videoView);
    }
}