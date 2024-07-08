package com.github.yoep.popcorn.backend.playlists.model;

import lombok.Builder;

import java.util.List;

import static java.util.Arrays.asList;

public record Playlist(List<PlaylistItem> items) {
    @Builder
    public Playlist(PlaylistItem... items) {
        this(asList(items));
    }
}
