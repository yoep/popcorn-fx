package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.Episode;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.ShowOverview;
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
public class PlaylistManager extends AbstractListenerService<PlaylistManagerListener> implements PlaylistManagerCallback {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ApplicationConfig applicationConfig;
    private Long playlistLoaderHandle;

    public PlaylistManager(FxLib fxLib, PopcornFx instance, ApplicationConfig applicationConfig) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.applicationConfig = applicationConfig;
        init();
    }

    public void play(Playlist.ByValue playlist) {
        try (playlist) {
            log.debug("Starting playback of playlist {}", playlist);
            playlistLoaderHandle = fxLib.play_playlist(instance, playlist);
        } catch (Exception ex) {
            log.error("Failed to start playlist, {}", ex.getMessage(), ex);
        }
    }

    public void play(MovieDetails movie, String quality) {
        Objects.requireNonNull(movie, "movie cannot be null");
        var items = new ArrayList<PlaylistItem>();
        items.add(itemFrom(movie, quality));
        play(new Playlist.ByValue(items.toArray(new PlaylistItem[0])));
    }

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

        play(new Playlist.ByValue(items.toArray(new PlaylistItem[0])));
    }

    public void playNext() {
        playlistLoaderHandle = fxLib.play_next_playlist_item(instance);
    }

    public void stop() {
        fxLib.stop_playlist(instance);
    }

    public Playlist playlist() {
        return fxLib.playlist(instance);
    }

    @Override
    public void callback(PlaylistManagerEvent.ByValue event) {
        try (event) {
            switch (event.getTag()) {
                case PlaylistChanged -> invokeListeners(PlaylistManagerListener::onPlaylistChanged);
                case PlayingNext -> {
                    var playingIn = event.getUnion().getPlayingNext_body();
                    invokeListeners(e -> e.onPlayingIn(playingIn.getPlayingIn().orElse(null), playingIn.getItem()));
                }
                case StateChanged -> invokeListeners(e -> e.onStateChanged(event.getUnion().getStateChanged_body().getState()));
            }
        }
    }

    private void init() {
        fxLib.register_playlist_manager_callback(instance, this);
    }

    private static PlaylistItem itemFrom(MovieDetails movie, String quality) {
        var item = new PlaylistItem();
        item.title = movie.getTitle();
        item.thumb = movie.getImages().getPoster();
        item.media = MediaItem.from(movie).toReference();
        item.quality = quality;
        item.setSubtitlesEnabled(true);
        return item;
    }

    private static PlaylistItem itemFrom(ShowOverview show, Episode episode, String quality) {
        var item = new PlaylistItem();
        item.title = show.getTitle();
        item.caption = episode.getTitle();
        item.thumb = show.getImages().getPoster();
        item.parentMedia = MediaItem.from(show).toReference();
        item.media = MediaItem.from(episode).toReference();
        item.quality = quality;
        item.setSubtitlesEnabled(true);
        return item;
    }

    private static boolean isEpisodeGreater(Episode original, Episode compare) {
        if (original.getSeason() < compare.getSeason()) {
            return true;
        }

        return original.getSeason() == compare.getSeason() && original.getEpisode() < compare.getEpisode();
    }
}
