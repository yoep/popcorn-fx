package com.github.yoep.popcorn.backend.playlists.ffi;

import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowOverview;
import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;

class PlaylistItemTest {

    @Test
    void testGetUrl_whenUrlIsNull() {
        var item = PlaylistItem.builder()
                .url(null)
                .build();

        assertEquals(Optional.empty(), item.getUrl());
    }

    @Test
    void testGetUrl_whenUrlIsNotNull() {
        var expectedResult = "https://localhost:98745/myurl";

        var item = PlaylistItem.builder()
                .url(expectedResult)
                .build();

        assertEquals(Optional.of(expectedResult), item.getUrl());
    }

    @Test
    void testGetMedia_whenMediaIsNull() {
        var item = PlaylistItem.builder()
                .media(null)
                .build();

        assertEquals(Optional.empty(), item.getMedia());
    }

    @Test
    void testGetMedia_whenMediaIsNotNull() {
        var media = new ShowOverview.ByReference();
        var item = PlaylistItem.builder()
                .media(MediaItem.from(media))
                .build();

        assertEquals(Optional.of(media), item.getMedia());
    }

    @Test
    void testGetParentMedia_whenParentMediaIsNull() {
        var item = PlaylistItem.builder()
                .parentMedia(null)
                .build();

        assertEquals(Optional.empty(), item.getParentMedia());
    }

    @Test
    void testGetParentMedia_whenParentMediaIsNotNull() {
        var media = new MovieDetails.ByReference();
        var item = PlaylistItem.builder()
                .parentMedia(MediaItem.from(media))
                .build();

        assertEquals(Optional.of(media), item.getParentMedia());
    }

    @Test
    void testClose() {
        var item = new PlaylistItem();
        item.close();
    }
}