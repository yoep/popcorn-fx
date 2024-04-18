package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlaylistManagerTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private PlaybackSettings playbackSettings;
    @InjectMocks
    private PlaylistManager playlistManager;

    @Test
    void testPlay_Playlist() {
        var playlist = new Playlist.ByValue();

        playlistManager.play(playlist);

        verify(fxLib).play_playlist(instance, playlist);
    }

    @Test
    void testPlay_Movie() {
        var playlistHolder = new AtomicReference<Playlist>();
        var movieTitle = "MyMovie";
        var movie = MovieDetails.builder()
                .title(movieTitle)
                .images(new Images())
                .build();
        var quality = "1080p";
        doAnswer(invocation -> {
            playlistHolder.set(invocation.getArgument(1, Playlist.class));
            return null;
        }).when(fxLib).play_playlist(isA(PopcornFx.class), isA(Playlist.ByValue.class));

        playlistManager.play(movie, quality);

        verify(fxLib).play_playlist(isA(PopcornFx.class), isA(Playlist.ByValue.class));
        var playlist = playlistHolder.get();
        var playlistItems = playlist.getCachedItems();
        assertEquals(1, playlistItems.size());
        assertEquals(quality, playlistItems.get(0).quality);
        assertEquals(movieTitle, playlistItems.get(0).title);
        assertEquals((byte) 1, playlistItems.get(0).subtitlesEnabled);
    }

    @Test
    void testPlay_Episode() {
        var playlistHolder = new AtomicReference<Playlist>();
        var showTitle = "MyShowTitle";
        var episodeTitle = "MyEpisode";
        var thumbnail = "ShowPosterUrl";
        var show = ShowDetails.builder()
                .title(showTitle)
                .images(Images.builder()
                        .poster(thumbnail)
                        .build())
                .build();
        var episode = Episode.builder()
                .title(episodeTitle)
                .episode(10)
                .build();
        var quality = "1080p";
        doAnswer(invocation -> {
            playlistHolder.set(invocation.getArgument(1, Playlist.class));
            return null;
        }).when(fxLib).play_playlist(isA(PopcornFx.class), isA(Playlist.ByValue.class));
        when(applicationConfig.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getPlaybackSettings()).thenReturn(playbackSettings);
        when(playbackSettings.isAutoPlayNextEpisodeEnabled()).thenReturn(true);

        playlistManager.play(show, episode, quality);

        verify(playbackSettings).isAutoPlayNextEpisodeEnabled();
        verify(fxLib).play_playlist(isA(PopcornFx.class), isA(Playlist.ByValue.class));
        var playlist = playlistHolder.get();
        var playlistItems = playlist.getCachedItems();
        assertEquals(1, playlistItems.size());
        assertEquals(showTitle, playlistItems.get(0).getTitle());
        assertEquals(Optional.of(episodeTitle), playlistItems.get(0).getCaption());
        assertEquals(thumbnail, playlistItems.get(0).thumb);
        assertEquals(quality, playlistItems.get(0).quality);
    }

    @Test
    void testPlayNext() {
        playlistManager.playNext();

        verify(fxLib).play_next_playlist_item(instance);
    }

    @Test
    void testStop() {
        playlistManager.stop();

        verify(fxLib).stop_playlist(instance);
    }

    @Test
    void testPlaylist() {
        var playlist = mock(Playlist.ByValue.class);
        when(fxLib.playlist(isA(PopcornFx.class))).thenReturn(playlist);

        var result = playlistManager.playlist();

        assertEquals(playlist, result);
        verify(fxLib).playlist(instance);
    }
}