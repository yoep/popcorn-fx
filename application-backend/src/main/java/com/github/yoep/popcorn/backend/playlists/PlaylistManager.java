package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;

import java.util.ArrayList;

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
        fxLib.play_playlist(instance, playlist);
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

    public void play(PlaylistItem item) {
        fxLib.play_playlist_item(instance, item);
    }

    private static PlaylistItem itemFrom(ShowOverview show, Episode episode, String quality) {
        var item = new PlaylistItem();
        item.url = episode.getTorrents().get(quality).url;
        item.title = episode.getTitle();
        item.thumb = show.getImages().getPoster();
        item.media = MediaItem.from(episode).toReference();
        item.quality = quality;
        return item;
    }

    private static boolean isEpisodeGreater(Episode original, Episode compare) {
        if (original.getSeason() < compare.getSeason()) {
            return true;
        }

        return original.getSeason() == compare.getSeason() && original.getEpisode() < compare.getEpisode();
    }
}
