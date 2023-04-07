package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.FxLibInstance;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"movies", "moviesLen", "shows", "showsLen"})
public class MediaSet extends Structure implements Closeable {
    public static class ByValue extends MediaSet implements Structure.ByValue {
    }

    public MovieOverview.ByReference movies;
    public int moviesLen;
    public ShowOverview.ByReference shows;
    public int showsLen;

    public List<MovieOverview> getMovies() {
        return Optional.ofNullable(movies)
                .map(e -> (MovieOverview[]) e.toArray(moviesLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    public List<ShowOverview> getShows() {
        return Optional.ofNullable(shows)
                .map(e -> (ShowOverview[]) e.toArray(showsLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    public static MediaSet from(List<Media> items) {
        var movies = items.stream()
                .filter(e -> e instanceof MovieOverview)
                .map(e -> (MovieOverview) e)
                .toList();
        var shows = items.stream()
                .filter(e -> e instanceof ShowOverview)
                .map(e -> (ShowOverview) e)
                .toList();
        var mediaSet = new MediaSet();

        if (movies.size() > 0)
            mediaSet.toMovieArray(movies);
        if (shows.size() > 0)
            mediaSet.toShowArray(shows);

        return mediaSet;
    }

    @Override
    public void close() {
        setAutoSynch(false);
        FxLibInstance.INSTANCE.get().dispose_media_items(this);
    }

    private void toMovieArray(List<MovieOverview> items) {
        this.movies = new MovieOverview.ByReference();
        this.moviesLen = items.size();

        var array = (MovieOverview.ByReference[]) this.movies.toArray(moviesLen);

        for (int i = 0; i < moviesLen; i++) {
            var item = items.get(i);
            array[i].title = item.title;
            array[i].imdbId = item.imdbId;
            array[i].year = item.year;
            array[i].rating = item.rating;
            array[i].images = item.images;
        }
    }

    private void toShowArray(List<ShowOverview> items) {
        this.shows = new ShowOverview.ByReference();
        this.showsLen = items.size();

        var array = (ShowOverview.ByReference[]) this.shows.toArray(showsLen);

        for (int i = 0; i < showsLen; i++) {
            var item = items.get(i);
            array[i].imdbId = item.imdbId;
            array[i].tvdbId = item.tvdbId;
            array[i].title = item.title;
            array[i].year = item.year;
            array[i].numberOfSeasons = item.numberOfSeasons;
            array[i].images = item.images;
            array[i].rating = item.rating;
        }
    }
}
