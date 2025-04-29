package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Images;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Rating;
import com.github.yoep.popcorn.backend.media.providers.MediaType;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

public record Season(int season, String text) implements Media, Comparable<Season> {
    @Override
    public String id() {
        return null;
    }

    @Override
    public String title() {
        return text;
    }

    @Override
    public String synopsis() {
        return null;
    }

    @Override
    public String year() {
        return null;
    }

    @Override
    public Integer runtime() {
        return null;
    }

    @Override
    public List<String> genres() {
        return Collections.emptyList();
    }

    @Override
    public Optional<Rating> getRating() {
        return Optional.empty();
    }

    @Override
    public Images images() {
        return null;
    }

    @Override
    public MediaType type() {
        return MediaType.SHOW;
    }

    @Override
    public String toString() {
        return text;
    }

    @Override
    public int compareTo(Season other) {
        return Integer.compare(season(), other.season());
    }
}
