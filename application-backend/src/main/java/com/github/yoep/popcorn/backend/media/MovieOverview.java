package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.providers.MediaType;
import lombok.Builder;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

public record MovieOverview(Media.MovieOverview proto) implements com.github.yoep.popcorn.backend.media.Media {
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
        return "";
    }

    @Override
    public String year() {
        return proto.getYear();
    }

    @Override
    public Integer runtime() {
        return 0;
    }

    @Override
    public List<String> genres() {
        return Collections.emptyList();
    }

    @Override
    public Optional<Media.Rating> getRating() {
        return Optional.ofNullable(proto.getRating());
    }

    @Override
    public Media.Images images() {
        return proto.getImages();
    }
}
