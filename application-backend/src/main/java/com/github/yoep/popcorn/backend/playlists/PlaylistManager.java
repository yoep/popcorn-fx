package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class PlaylistManager extends AbstractListenerService<PlaylistManagerListener> {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ApplicationConfig applicationConfig;

    public PlaylistManager(FxLib fxLib, PopcornFx instance, ApplicationConfig applicationConfig) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.applicationConfig = applicationConfig;
    }

    public void play(Playlist playlist) {
        try (playlist) {
            fxLib.play_playlist(instance, playlist);
        } catch (Exception ex) {
            log.error("Failed to start playlist, {}", ex.getMessage(), ex);
        }
    }

    public void play(MovieDetails movie, String quality) {
        var items = new ArrayList<PlaylistItem>();
        items.add(itemFrom(movie, quality));
        play(new Playlist(items));
    }

    public void play(ShowDetails show, Episode episode, String quality) {
        var items = new ArrayList<PlaylistItem>();

        items.add(itemFrom(show, episode, quality));

        if (applicationConfig.getSettings().playbackSettings.isAutoPlayNextEpisodeEnabled()) {
            var sortedEpisodes = show.getEpisodes().stream()
                    .filter(e -> isEpisodeGreater(episode, e))
                    .sorted()
                    .toList();

            for (Episode e : sortedEpisodes) {
                items.add(itemFrom(show, e, quality));
            }
        }

        play(new Playlist(items));
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
        item.title = episode.getTitle();
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
