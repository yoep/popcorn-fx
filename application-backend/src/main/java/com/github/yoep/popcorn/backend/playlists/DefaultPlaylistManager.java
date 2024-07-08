package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.Episode;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.ShowOverview;
import com.github.yoep.popcorn.backend.playlists.model.Playlist;
import com.github.yoep.popcorn.backend.playlists.model.PlaylistItem;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;
import java.util.Objects;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class DefaultPlaylistManager extends AbstractListenerService<PlaylistManagerListener> implements PlaylistManagerCallback, PlaylistManager {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ApplicationConfig applicationConfig;
    private Long playlistLoaderHandle;

    public DefaultPlaylistManager(FxLib fxLib, PopcornFx instance, ApplicationConfig applicationConfig) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.applicationConfig = applicationConfig;
        init();
    }

    @Override
    public void play(Playlist playlist) {
        var items = playlist.items().stream()
                .map(com.github.yoep.popcorn.backend.playlists.ffi.PlaylistItem::from)
                .toArray(com.github.yoep.popcorn.backend.playlists.ffi.PlaylistItem[]::new);

        var playlist_c = new com.github.yoep.popcorn.backend.playlists.ffi.Playlist.ByReference(items);
        try (playlist_c) {
            log.debug("Starting playback of playlist {}", playlist_c);
            playlistLoaderHandle = fxLib.play_playlist(instance, playlist_c);
        } catch (Exception ex) {
            log.error("Failed to start playlist, {}", ex.getMessage(), ex);
        }
        // data will be cleaned by JNA as the memory was assigned by JNA, don't try to clean it here
    }

    @Override
    public void play(MovieDetails movie, String quality) {
        Objects.requireNonNull(movie, "movie cannot be null");
        var items = new ArrayList<PlaylistItem>();
        items.add(itemFrom(movie, quality));
        play(new Playlist(items.toArray(new PlaylistItem[0])));
    }

    @Override
    public void play(ShowDetails show, Episode episode, String quality) {
        Objects.requireNonNull(show, "show cannot be null");
        Objects.requireNonNull(episode, "episode cannot be null");
        var items = new ArrayList<PlaylistItem>();

        items.add(itemFrom(show, episode, quality));

        if (applicationConfig.getSettings().getPlaybackSettings().isAutoPlayNextEpisodeEnabled()) {
            var sortedEpisodes = show.getEpisodes().stream()
                    .filter(e -> isEpisodeGreater(episode, e))
                    .sorted()
                    .toList();

            for (Episode e : sortedEpisodes) {
                items.add(itemFrom(show, e, quality));
            }
        }

        play(new Playlist(items.toArray(new PlaylistItem[0])));
    }

    @Override
    public void playNext() {
        playlistLoaderHandle = fxLib.play_next_playlist_item(instance);
    }

    @Override
    public void stop() {
        fxLib.stop_playlist(instance);
    }

    @Override
    public Playlist playlist() {
        var playlist = fxLib.playlist(instance);
        try (playlist) {
            return new Playlist(playlist.getItems().stream()
                    .map(PlaylistItem::from)
                    .toArray(PlaylistItem[]::new));
        } finally {
            fxLib.dispose_playlist_set(playlist);
        }
    }

    @Override
    public void callback(PlaylistManagerEvent.ByValue event) {
        try (event) {
            switch (event.getTag()) {
                case PlaylistChanged -> invokeListeners(PlaylistManagerListener::onPlaylistChanged);
                case PlayingNext -> {
                    var playingIn = event.getUnion().getPlayingNext_body();
                    var item = PlaylistItem.from(playingIn.getItem());
                    invokeListeners(e -> e.onPlayingIn(playingIn.getPlayingIn().orElse(null), item));
                }
                case StateChanged -> invokeListeners(e -> e.onStateChanged(event.getUnion().getStateChanged_body().getState()));
            }
        }
    }

    private void init() {
        fxLib.register_playlist_manager_callback(instance, this);
    }

    private static PlaylistItem itemFrom(MovieDetails movie, String quality) {
        return PlaylistItem.builder()
                .title(movie.getTitle())
                .caption(movie.getTitle())
                .thumb(movie.getImages().getPoster())
                .media(MediaItem.from(movie).toReference())
                .quality(quality)
                .subtitlesEnabled(true)
                .build();
    }

    private static PlaylistItem itemFrom(ShowOverview show, Episode episode, String quality) {
        return PlaylistItem.builder()
                .title(show.getTitle())
                .caption(episode.getTitle())
                .thumb(show.getImages().getPoster())
                .parentMedia(MediaItem.from(show).toReference())
                .media(MediaItem.from(episode).toReference())
                .quality(quality)
                .subtitlesEnabled(true)
                .build();
    }

    private static boolean isEpisodeGreater(Episode original, Episode compare) {
        if (original.getSeason() < compare.getSeason()) {
            return true;
        }

        return original.getSeason() == compare.getSeason() && original.getEpisode() < compare.getEpisode();
    }
}
