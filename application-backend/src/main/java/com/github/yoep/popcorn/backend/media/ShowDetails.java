package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.providers.MediaType;

import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public record ShowDetails(Media.ShowDetails proto) implements com.github.yoep.popcorn.backend.media.Media {
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
        return proto.getGenreList();
    }

    @Override
    public Optional<Media.Rating> getRating() {
        return Optional.ofNullable(proto.getRating());
    }

    @Override
    public Media.Images images() {
        return proto.getImages();
    }

    @Override
    public MediaType type() {
        return MediaType.SHOW;
    }

    public List<Episode> getEpisodes() {
        return proto.getEpisodesList().stream()
                .map(Episode::new)
                .collect(Collectors.toList());
    }
}
