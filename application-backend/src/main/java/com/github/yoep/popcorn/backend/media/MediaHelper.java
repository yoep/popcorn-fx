package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Item;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.MovieOverview;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.ShowOverview;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.MovieDetails;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.ShowDetails;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Episode;
import com.google.protobuf.MessageLite;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class MediaHelper {
    public static Media getMedia(Item item) {
        var type = item.getType();
        if (Objects.equals(type, FxChannel.typeFrom(MovieOverview.class))) {
            return new com.github.yoep.popcorn.backend.media.MovieOverview(item.getMovieOverview());
        } else if (Objects.equals(type, FxChannel.typeFrom(ShowOverview.class))) {
            return new com.github.yoep.popcorn.backend.media.ShowOverview(item.getShowOverview());
        } else if (Objects.equals(type, FxChannel.typeFrom(MovieDetails.class))) {
            return new com.github.yoep.popcorn.backend.media.MovieDetails(item.getMovieDetails());
        } else if (Objects.equals(type, FxChannel.typeFrom(ShowDetails.class))) {
            return new com.github.yoep.popcorn.backend.media.ShowDetails(item.getShowDetails());
        } else if (Objects.equals(type, FxChannel.typeFrom(Episode.class))) {
            return new com.github.yoep.popcorn.backend.media.Episode(item.getEpisode());
        }

        throw new MediaException(String.format("invalid media type %s", type));
    }

    public static <T extends MessageLite> T getProto(Media media) {
        if (media instanceof com.github.yoep.popcorn.backend.media.MovieOverview e) {
            return (T) e.proto();
        } else if (media instanceof com.github.yoep.popcorn.backend.media.ShowOverview e) {
            return (T) e.proto();
        } else if (media instanceof com.github.yoep.popcorn.backend.media.MovieDetails e) {
            return (T) e.proto();
        } else if (media instanceof com.github.yoep.popcorn.backend.media.ShowDetails e) {
            return (T) e.proto();
        } else if (media instanceof com.github.yoep.popcorn.backend.media.Episode e) {
            return (T) e.proto();
        }

        throw new MediaException(String.format("invalid media %s", media.getClass().getSimpleName()));
    }

    public static Item getItem(Media media) {
        var itemBuilder = Item.newBuilder();
        var proto = getProto(media);

        if (proto instanceof MovieOverview e) {
            itemBuilder.setType(FxChannel.typeFrom(MovieOverview.class));
            itemBuilder.setMovieOverview(e);
        } else if (proto instanceof ShowOverview e) {
            itemBuilder.setType(FxChannel.typeFrom(ShowOverview.class));
            itemBuilder.setShowOverview(e);
        } else if (proto instanceof MovieDetails e) {
            itemBuilder.setType(FxChannel.typeFrom(MovieDetails.class));
            itemBuilder.setMovieDetails(e);
        } else if (proto instanceof ShowDetails e) {
            itemBuilder.setType(FxChannel.typeFrom(ShowDetails.class));
            itemBuilder.setShowDetails(e);
        } else if (proto instanceof Episode e) {
            itemBuilder.setType(FxChannel.typeFrom(Episode.class));
            itemBuilder.setEpisode(e);
        }

        return itemBuilder.build();
    }
}
