package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class MediaHelperTest {
    @Test
    void testGetMedia_movieOverview() {
        var item = Media.Item.newBuilder()
                .setType(FxChannel.typeFrom(Media.MovieOverview.class))
                .setMovieOverview(Media.MovieOverview.getDefaultInstance())
                .build();

        var media = MediaHelper.getMedia(item);

        assertInstanceOf(MovieOverview.class, media, "expected MovieOverview");
    }

    @Test
    void testGetMedia_showOverview() {
        var item = Media.Item.newBuilder()
                .setType(FxChannel.typeFrom(Media.ShowOverview.class))
                .setShowOverview(Media.ShowOverview.getDefaultInstance())
                .build();

        var media = MediaHelper.getMedia(item);

        assertInstanceOf(ShowOverview.class, media, "expected ShowOverview");
    }
}