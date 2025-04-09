package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.providers.MediaType;

import java.util.List;
import java.util.Optional;

public record MovieDetails(Media.MovieDetails proto) implements com.github.yoep.popcorn.backend.media.Media {
    @Override
    public MediaType type() {
        return MediaType.MOVIE;
    }

    @Override
    public String id() {
        return proto.getImdbId();
    }

    @Override
    public String title() {
        return proto.getTitle();
    }

    @Override
    public String synopsis() {
        return proto.getSynopsis();
    }

    @Override
    public String year() {
        return proto.getYear();
    }

    @Override
    public Integer runtime() {
        return proto.getRuntime();
    }

    @Override
    public List<String> genres() {
        return proto.getGenresList();
    }

    @Override
    public Optional<Media.Rating> getRating() {
        return Optional.ofNullable(proto.getRating());
    }

    @Override
    public Media.Images images() {
        return proto.getImages();
    }

    public List<Media.TorrentLanguage> getTorrents() {
        return proto.getTorrentsList();
    }
}
