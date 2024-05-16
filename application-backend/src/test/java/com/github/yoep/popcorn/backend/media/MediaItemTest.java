package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.media.providers.Episode;
import com.github.yoep.popcorn.backend.media.providers.ShowOverview;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class MediaItemTest {
    @Test
    void testToReference() {
        var show = new ShowOverview.ByReference();
        show.title = "FooBar";
        var media = MediaItem.from(show);

        var result = media.toReference();

        assertEquals(show, result.showOverview);
    }

    @Test
    void testGetMedia() {
        var episode = new Episode.ByReference();
        episode.title = "FooBar";
        episode.episode = 2;
        episode.season = 1;
        var media = MediaItem.from(episode);

        var result = media.getMedia();

        assertEquals(episode, result);
    }
}