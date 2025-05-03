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

import java.util.ArrayList;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
public class DefaultPlaylistManager extends AbstractListenerService<PlaylistManagerListener> implements PlaylistManager {
    private final FxChannel fxChannel;
    private final ApplicationConfig applicationConfig;

    AtomicReference<Handle> playlistLoaderHandle = new AtomicReference<>();

    public DefaultPlaylistManager(FxChannel fxChannel, ApplicationConfig applicationConfig) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        Objects.requireNonNull(applicationConfig, "applicationConfig cannot be null");
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
        var items = new ArrayList<Playlist.Item>();

        items.add(playlistItemFrom(show, episode, quality));

        applicationConfig.getSettings()
                .thenApply(settings -> {
                    if (settings.getPlaybackSettings().getAutoPlayNextEpisodeEnabled()) {
                        var sortedEpisodes = show.getEpisodes().stream()
                                .filter(e -> isEpisodeGreater(episode, e))
                                .sorted()
                                .toList();

                        for (Episode e : sortedEpisodes) {
                            items.add(playlistItemFrom(show, e, quality));
                        }
                    }

                    return items;
                })
                .whenComplete((result, throwable) -> {
                    if (throwable == null) {
                        play(Playlist.newBuilder()
                                .addAllItems(result)
                                .build());
                    } else {
                        log.error("Failed to retrieve settings", throwable);
                    }
                });
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
    public CompletableFuture<Playlist> playlist() {
        return fxChannel.send(GetActivePlaylistRequest.getDefaultInstance(), GetActivePlaylistResponse.parser())
                .thenApply(GetActivePlaylistResponse::getPlaylist);
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(PlaylistEvent.class), PlaylistEvent.parser(), this::onPlaylistEvent);
    }

    private void onPlaylistEvent(PlaylistEvent event) {
        switch (event.getEvent()) {
            case PLAYLIST_CHANGED -> invokeListeners(PlaylistManagerListener::onPlaylistChanged);
            case PLAYING_NEXT -> invokeListeners(e -> e.onPlayingIn(event.getPlayingNext().getPlayingIn(), event.getPlayingNext().getItem()));
            case STATE_CHANGED -> invokeListeners(e -> e.onStateChanged(event.getStateChanged().getState()));
        }
    }

    private static Playlist.Item playlistItemFrom(ShowDetails show, Episode episode, String quality) {
        return Playlist.Item.newBuilder()
                .setTitle(show.title())
                .setCaption(episode.title())
                .setThumb(show.images().getPoster())
                .setParentMedia(MediaHelper.getItem(show))
                .setMedia(MediaHelper.getItem(episode))
                .setQuality(quality)
                .setSubtitlesEnabled(true)
                .build();
    }

    private static boolean isEpisodeGreater(Episode original, Episode compare) {
        if (original.season() < compare.season()) {
            return true;
        }

        return original.season() == compare.season() && original.episode() < compare.episode();
    }
}
