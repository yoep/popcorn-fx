package com.github.yoep.popcorn.trakt.models;

import com.github.yoep.popcorn.providers.models.MediaType;
import com.github.yoep.popcorn.watched.models.AbstractWatchable;
import com.github.yoep.popcorn.watched.models.Watchable;
import lombok.*;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class TraktMovie extends AbstractWatchable implements Watchable {
    private String title;
    private int year;
    private TraktMovieIds ids;

    @Override
    public String getId() {
        return ids.getImdb();
    }

    @Override
    public MediaType getType() {
        return MediaType.MOVIE;
    }
}
