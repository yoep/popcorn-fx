package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import javax.validation.constraints.NotNull;
import java.io.Closeable;
import java.io.IOException;
import java.util.Collection;
import java.util.List;
import java.util.Optional;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
@Structure.FieldOrder({"id", "imdbId", "tvdbId", "title", "year", "runtime", "numberOfSeasons", "images", "rating", "synopsis"})
public class Show extends AbstractMedia implements Closeable {
    public static class ByReference extends Show implements Structure.ByReference {
    }

    public String tvdbId;
    public int numberOfSeasons;

    private String status;
    private List<Episode> episodes;

    private long lastUpdated;

    public Show() {
    }

    @Builder
    public Show(String id, String imdbId, String title, String year, Integer runtime, List<String> genres, Rating rating, Images images, String synopsis,
                String tvdbId, int numberOfSeasons, String status, List<Episode> episodes, long lastUpdated) {
        super(id, imdbId, title, year, runtime, genres, toRatingReference(rating), images, synopsis);
        this.tvdbId = tvdbId;
        this.numberOfSeasons = numberOfSeasons;
        this.status = status;
        this.lastUpdated = lastUpdated;

        setEpisodes(episodes);
    }

    /**
     * Create a copy of the given show.
     *
     * @param show The show to copy.
     */
    public Show(@NotNull Show show) {
        super(show.getId(), show.getImdbId(), show.getTitle(), show.getYear(), show.getRuntime(), show.getGenres(),
                toRatingReference(show.getRating().orElse(null)),
                show.getImages(), show.getSynopsis());
        this.tvdbId = show.tvdbId;
        this.numberOfSeasons = show.getNumberOfSeasons();
        this.status = show.getStatus();
        this.lastUpdated = show.getLastUpdated();
        this.episodes = Optional.ofNullable(show.getEpisodes()).stream()
                .flatMap(Collection::stream)
                .map(Episode::new)
                .toList();
    }

    /**
     * Set the episodes of the show.
     *
     * @param episodes The available episodes for the show.
     */
    public void setEpisodes(List<Episode> episodes) {
        this.episodes = episodes;
    }

    @Override
    @JsonIgnore
    public MediaType getType() {
        return MediaType.SHOW;
    }

    @Override
    public void close() throws IOException {
        setAutoSynch(false);
    }
}
