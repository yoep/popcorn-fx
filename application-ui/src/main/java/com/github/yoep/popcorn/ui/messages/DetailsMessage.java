package com.github.yoep.popcorn.ui.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum DetailsMessage implements Message {
    MAGNET_LINK("details_magnet_link"),
    MAGNET_LINK_COPIED_TO_CLIPBOARD("details_copied_magnet_link"),
    MAGNET_LINK_OPENING("details_magnet_link_opening"),
    MAGNET_LINK_FAILED_TO_OPEN("details_magnet_link_failed_to_open"),
    SEASON("details_season"),
    ADD_TO_BOOKMARKS("details_add_to_bookmarks"),
    REMOVE_FROM_BOOKMARKS("details_remove_from_bookmarks"),
    FAVORITE("details_favorite"),
    UNFAVORED("details_unfavored"),
    NOT_SEEN("details_not_seen"),
    SEEN("details_seen"),
    MARK_AS_NOT_SEEN("details_mark_as_not_seen"),
    MARK_AS_SEEN("details_marks_as_seen"),
    EPISODE_SEASON("details_episode_season"),
    AIR_DATE("details_air_date"),
    MARK_AS_WATCHED("details_mark_as_watched"),
    UNMARK_AS_WATCHED("details_unmark_as_watched"),
    DETAILS_FAILED_TO_LOAD("details_failed_to_load");

    private final String key;

    DetailsMessage(String key) {
        this.key = key;
    }
}
