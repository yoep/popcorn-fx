package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class PlaylistManagerTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private ApplicationConfig applicationConfig;
    @InjectMocks
    private PlaylistManager playlistManager;

    @Test
    void testPlay_Playlist() {
        var playlist = new Playlist();

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
        }).when(fxLib).play_playlist(isA(PopcornFx.class), isA(Playlist.class));

        playlistManager.play(movie, quality);

        verify(fxLib).play_playlist(isA(PopcornFx.class), isA(Playlist.class));
        var playlist = playlistHolder.get();
        var playlistItems = playlist.getCachedItems();
        assertEquals(1, playlistItems.size());
        assertEquals(quality, playlistItems.get(0).quality);
        assertEquals(movieTitle, playlistItems.get(0).title);
        assertEquals((byte) 1, playlistItems.get(0).subtitlesEnabled);
    }
}