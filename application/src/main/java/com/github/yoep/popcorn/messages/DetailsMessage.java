package com.github.yoep.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum DetailsMessage implements Message {
    MAGNET_LINK("details_magnet_link"),
    SEASON("details_season"),
    ADD_TO_BOOKMARKS("details_add_to_bookmarks"),
    REMOVE_FROM_BOOKMARKS("details_remove_from_bookmarks"),
    NOT_SEEN("details_not_seen"),
    SEEN("details_seen"),
    EPISODE_SEASON("details_episode_season"),
    AIR_DATE("details_air_date"),
    MARK_AS_WATCHED("details_mark_as_watched"),
    UNMARK_AS_WATCHED("details_unmark_as_watched");

    private final String key;

    DetailsMessage(String key) {
        this.key = key;
    }
}
