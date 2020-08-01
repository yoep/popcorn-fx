package com.github.yoep.popcorn.ui.trakt.models;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.github.yoep.popcorn.ui.trakt.TraktDateTimeDeserializer;
import lombok.Data;

import java.time.LocalDateTime;

@Data
public class WatchListItem {
    private int rank;
    @JsonProperty("listed_at")
    @JsonDeserialize(using = TraktDateTimeDeserializer.class)
    private LocalDateTime listedAt;
    private TraktType type;
    private TraktMovie movie;
    private TraktShow show;
}
