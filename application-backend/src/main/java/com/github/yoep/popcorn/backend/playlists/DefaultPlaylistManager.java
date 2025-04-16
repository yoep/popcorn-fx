package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class DefaultPlaylistManager extends AbstractListenerService<PlaylistManagerListener> implements PlaylistManager {
    private final FxChannel fxChannel;
    private final ApplicationConfig applicationConfig;

    private AtomicReference<Handle> playlistLoaderHandle = new AtomicReference<>();

    public DefaultPlaylistManager(FxChannel fxChannel, ApplicationConfig applicationConfig) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        this.applicationConfig = applicationConfig;
        init();
    }

    @Override
    public void play(Playlist playlist) {
        Objects.requireNonNull(playlist, "playlist cannot be null");
        fxChannel.send(
                        PlayPlaylistRequest.newBuilder()
                                .setPlaylist(playlist)
                                .build(), PlayPlaylistResponse.parser()
                )
                .thenApply(PlayPlaylistResponse::getHandle)
                .whenComplete((handle, throwable) -> {
                    if (throwable == null) {
                        playlistLoaderHandle.set(handle);
                    } else {
                        log.error("Failed to play playlist", throwable);
                    }
                });
    }

    @Override
    public void play(MovieDetails movie, String quality) {
        Objects.requireNonNull(movie, "movie cannot be null");
        play(Playlist.newBuilder()
                .addItems(Playlist.Item.newBuilder()
                        .setTitle(movie.title())
                        .setCaption(quality)
                        .setThumb(movie.proto().getImages().getPoster())
                        .setMedia(MediaHelper.getItem(movie))
                        .setQuality(quality)
                        .setSubtitlesEnabled(true)
                        .build())
                .build());
    }

    @Override
    public void play(ShowDetails show, Episode episode, String quality) {
        Objects.requireNonNull(show, "show cannot be null");
        Objects.requireNonNull(episode, "episode cannot be null");
//        var items = new ArrayList<PlaylistItem>();
//
//        items.add(itemFrom(show, episode, quality));
//
//        if (applicationConfig.getSettings().getPlaybackSettings().getAutoPlayNextEpisodeEnabled()) {
//            var sortedEpisodes = show.getEpisodes().stream()
//                    .filter(e -> isEpisodeGreater(episode, e))
//                    .sorted()
//                    .toList();
//
//            for (Episode e : sortedEpisodes) {
//                items.add(itemFrom(show, e, quality));
//            }
//        }
//
//        play(new Playlist(items.toArray(new PlaylistItem[0])));
    }

    @Override
    public void playNext() {
        fxChannel.send(PlayNextPlaylistItemRequest.getDefaultInstance(), PlayNextPlaylistItemResponse.parser())
                .thenApply(PlayNextPlaylistItemResponse::getHandle)
                .whenComplete((handle, throwable) -> {
                    if (throwable == null) {
                        playlistLoaderHandle.set(handle);
                    } else {
                        log.error("Failed to play next item", throwable);
                    }
                });

    }

    @Override
    public void stop() {
        fxChannel.send(StopPlaylistRequest.getDefaultInstance());
    }

    @Override
    public Playlist playlist() {
        // TODO
        return null;
    }
//
//    @Override
//    public void callback(PlaylistManagerEvent.ByValue event) {
//        try (event) {
//            switch (event.getTag()) {
//                case PlaylistChanged -> invokeListeners(PlaylistManagerListener::onPlaylistChanged);
//                case PlayingNext -> {
//                    var playingIn = event.getUnion().getPlayingNext_body();
//                    var item = PlaylistItem.from(playingIn.getItem());
//                    invokeListeners(e -> e.onPlayingIn(playingIn.getPlayingIn().orElse(null), item));
//                }
//                case StateChanged -> invokeListeners(e -> e.onStateChanged(event.getUnion().getStateChanged_body().getState()));
//            }
//        }
//    }

    private void init() {
        // TODO
    }

    private static boolean isEpisodeGreater(Episode original, Episode compare) {
        if (original.season() < compare.season()) {
            return true;
        }

        return original.season() == compare.season() && original.episode() < compare.episode();
    }
}
