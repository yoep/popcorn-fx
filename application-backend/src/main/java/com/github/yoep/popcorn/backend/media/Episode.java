package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.media.providers.MediaType;

import java.time.Instant;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

public record Episode(Media.Episode proto) implements Comparable<Episode>, com.github.yoep.popcorn.backend.media.Media {
    @Override
    public MediaType type() {
        return MediaType.SHOW;
    }

    @Override
    public String id() {
        return proto.getTvdbId();
    }

    public int season() {
        return proto.getSeason();
    }

    public int episode() {
        return proto.getEpisode();
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
    public Media.Images images() {
        return null;
    }

    /**
     * Get the local date time of the air date from this episode.
     *
     * @return Returns the air date as local date time.
     */
    public LocalDateTime getAirDate() {
        return LocalDateTime.ofInstant(Instant.ofEpochSecond(proto.getFirstAired()), ZoneOffset.UTC);
    }

    public Optional<String> getThumb() {
        return Optional.ofNullable(proto.getThumb());
    }

    @Override
    public String year() {
        return String.valueOf(getAirDate().getYear());
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
        return Optional.empty();
    }

    public Media.TorrentQuality getTorrents() {
        return proto.getTorrents();
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof Episode episode)) return false;

        return Objects.equals(id(), episode.id());
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(id());
    }

    @Override
    public int compareTo(Episode compareTo) {
        // order first by season
        var seasonCompareResult = Integer.compare(season(), compareTo.season());

        // if the seasons don't match
        // return the compare result
        if (seasonCompareResult != 0) {
            return seasonCompareResult;
        }

        // order by episode number
        return Integer.compare(episode(), compareTo.episode());
    }
}
