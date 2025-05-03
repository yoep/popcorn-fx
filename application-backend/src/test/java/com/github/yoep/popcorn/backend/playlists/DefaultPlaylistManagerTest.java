package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class DefaultPlaylistManagerTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private ApplicationConfig applicationConfig;
    @InjectMocks
    private DefaultPlaylistManager playlistManager;

    @Test
    void testPlay_Playlist() {
        var url = "http://localhost/example-video.mp4";
        var playlist = Playlist.newBuilder()
                .addItems(Playlist.Item.newBuilder()
                        .setUrl(url)
                        .build())
                .build();
        var request = new AtomicReference<PlayPlaylistRequest>();
        var response = PlayPlaylistResponse.newBuilder()
                .setHandle(Handle.newBuilder().setHandle(1212L).build())
                .build();
        when(fxChannel.send(isA(PlayPlaylistRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, PlayPlaylistRequest.class));
            return CompletableFuture.completedFuture(response);
        });

        playlistManager.play(playlist);

        verify(fxChannel).send(isA(PlayPlaylistRequest.class), isA(Parser.class));
        assertEquals(1212L, playlistManager.playlistLoaderHandle.get().getHandle());
        var result = request.get();
        assertNotNull(result, "expected a request to have been sent");
        assertEquals(url, result.getPlaylist().getItemsList().getFirst().getUrl());
    }

    @Test
    void testPlay_Movie() {
        var movieTitle = "MyMovie";
        var movie = new MovieDetails(Media.MovieDetails.newBuilder()
                .setTitle(movieTitle)
                .setImages(Media.Images.getDefaultInstance())
                .build());
        var quality = "1080p";
        var request = new AtomicReference<PlayPlaylistRequest>();
        var response = PlayPlaylistResponse.newBuilder()
                .setHandle(Handle.newBuilder().setHandle(100L).build())
                .build();
        when(fxChannel.send(isA(PlayPlaylistRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, PlayPlaylistRequest.class));
            return CompletableFuture.completedFuture(response);
        });

        playlistManager.play(movie, quality);

        verify(fxChannel).send(isA(PlayPlaylistRequest.class), isA(Parser.class));
        var resultItem = request.get().getPlaylist().getItemsList().getFirst();
        assertEquals(quality, resultItem.getQuality());
    }

    @Test
    void testPlay_Episode_AutoPlayNextDisabled() {
        var show = new ShowDetails(createShowWithEpisodes());
        var quality = "1080p";
        var request = new AtomicReference<PlayPlaylistRequest>();
        var response = PlayPlaylistResponse.newBuilder()
                .setHandle(Handle.newBuilder().setHandle(100L).build())
                .build();
        when(fxChannel.send(isA(PlayPlaylistRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, PlayPlaylistRequest.class));
            return CompletableFuture.completedFuture(response);
        });
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setPlaybackSettings(ApplicationSettings.PlaybackSettings.newBuilder()
                        .setAutoPlayNextEpisodeEnabled(false)
                        .build())
                .build()));

        playlistManager.play(show, show.getEpisodes().get(1), quality);

        verify(fxChannel).send(isA(PlayPlaylistRequest.class), isA(Parser.class));
        var resultItem = request.get().getPlaylist().getItemsList().getFirst();
        assertEquals(1, request.get().getPlaylist().getItemsList().size());
        assertEquals(show.title(), resultItem.getTitle());
        assertEquals("PlayingEpisode", resultItem.getCaption());
        assertEquals("http://some-poster-url", resultItem.getThumb());
        assertEquals(quality, resultItem.getQuality());
    }

    @Test
    void testPlay_Episode_AutoPlayNextEnabled() {
        var show = new ShowDetails(createShowWithEpisodes());
        var quality = "720p";
        var request = new AtomicReference<PlayPlaylistRequest>();
        when(fxChannel.send(isA(PlayPlaylistRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, PlayPlaylistRequest.class));
            return CompletableFuture.completedFuture(PlayPlaylistResponse.newBuilder()
                    .setHandle(Handle.newBuilder()
                            .setHandle(11L)
                            .build())
                    .build());
        });
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setPlaybackSettings(ApplicationSettings.PlaybackSettings.newBuilder()
                        .setAutoPlayNextEpisodeEnabled(true)
                        .build())
                .build()));

        playlistManager.play(show, show.getEpisodes().get(1), quality);

        verify(fxChannel).send(isA(PlayPlaylistRequest.class), isA(Parser.class));

        var currentItem = request.get().getPlaylist().getItemsList().getFirst();
        assertEquals(2, request.get().getPlaylist().getItemsList().size());
        assertEquals(show.title(), currentItem.getTitle());
        assertEquals("PlayingEpisode", currentItem.getCaption());
        assertEquals("http://some-poster-url", currentItem.getThumb());
        assertEquals(quality, currentItem.getQuality());

        var nextItem = request.get().getPlaylist().getItemsList().get(1);
        assertEquals(show.title(), nextItem.getTitle());
        assertEquals("NextEpisode", nextItem.getCaption());
        assertEquals("http://some-poster-url", nextItem.getThumb());
    }

    @Test
    void testPlayNext() {
        var handle = 33L;
        var request = new AtomicReference<PlayNextPlaylistItemRequest>();
        when(fxChannel.send(isA(PlayNextPlaylistItemRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, PlayNextPlaylistItemRequest.class));
            return CompletableFuture.completedFuture(PlayNextPlaylistItemResponse.newBuilder()
                    .setHandle(Handle.newBuilder()
                            .setHandle(handle)
                            .build())
                    .build());
        });

        playlistManager.playNext();

        verify(fxChannel).send(isA(PlayNextPlaylistItemRequest.class), isA(Parser.class));
        assertEquals(handle, playlistManager.playlistLoaderHandle.get().getHandle(), "expected the handle to have been updated");
    }

    @Test
    void testStop() {
        playlistManager.stop();

        verify(fxChannel).send(isA(StopPlaylistRequest.class));
    }

    @Test
    void testPlaylist() {
        var url = "http://localhost/example-video.mp4";
        var playlist = Playlist.newBuilder()
                .addItems(Playlist.Item.newBuilder()
                        .setUrl(url)
                        .setMedia(Media.Item.newBuilder()
                                .setShowDetails(createShowWithEpisodes())
                                .build())
                        .build())
                .build();
        when(fxChannel.send(isA(GetActivePlaylistRequest.class), isA(Parser.class)))
                .thenReturn(CompletableFuture.completedFuture(GetActivePlaylistResponse.newBuilder()
                        .setPlaylist(playlist)
                        .build()));

        var result = playlistManager.playlist().resultNow();
        assertEquals(1, result.getItemsList().size());

        var resultItem = result.getItemsList().getFirst();
        assertNotNull(resultItem, "expected a playlist item to have been present");
        assertEquals(url, resultItem.getUrl());
    }

    private static Media.ShowDetails createShowWithEpisodes() {
        return Media.ShowDetails.newBuilder()
                .setTitle("TheShowTitle")
                .setNumberOfSeasons(1)
                .setImages(Media.Images.newBuilder()
                        .setPoster("http://some-poster-url")
                        .setBanner("http://some-banner-url")
                        .setFanart("http://some-fanart-url")
                        .build())
                .addAllEpisodes(asList(
                        Media.Episode.newBuilder()
                                .setTitle("PreviousEpisode")
                                .setEpisode(9)
                                .setSeason(1)
                                .build(),
                        Media.Episode.newBuilder()
                                .setTitle("PlayingEpisode")
                                .setEpisode(10)
                                .setSeason(1)
                                .build(),
                        Media.Episode.newBuilder()
                                .setTitle("NextEpisode")
                                .setEpisode(11)
                                .setSeason(1)
                                .build()
                ))
                .build();
    }
}