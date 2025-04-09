package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Rating;
import com.github.yoep.popcorn.backend.media.providers.MediaType;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public record ShowOverview(Media.ShowOverview proto) implements com.github.yoep.popcorn.backend.media.Media {
    @Override
    public MediaType type() {
        return MediaType.SHOW;
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
    public Optional<Rating> getRating() {
        return Optional.ofNullable(proto.getRating());
    }

    @Override
    public Media.Images images() {
        return proto.getImages();
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

    public int getSeasons() {
        return proto.getNumberOfSeasons();
    }

    @Override
    public List<String> genres() {
        return new ArrayList<>();
    }
}
