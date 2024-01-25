package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.*;
import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Data
@ToString
@EqualsAndHashCode(callSuper = false)
@NoArgsConstructor
@Structure.FieldOrder({"url", "title", "thumb", "quality", "media", "autoResumeTimestamp"})
public class PlaylistItem extends Structure implements Closeable {
    public static class ByReference extends PlaylistItem implements Structure.ByReference {
    }

    public String url;
    public String title;
    public String thumb;
    public String quality;
    public MediaItem.ByReference media;
    public Long autoResumeTimestamp;

    public PlaylistItem(String url, String title, String thumb, MediaItem.ByReference media) {
        this.url = url;
        this.title = title;
        this.thumb = thumb;
        this.media = media;
    }

    public Optional<String> getUrl() {
        return Optional.ofNullable(url);
    }

    public Optional<Media> getMedia() {
        return Optional.ofNullable(media)
                .map(MediaItem::getMedia);
    }

    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    public static PlaylistItem fromMedia(Media media, String quality) {
        var item = new PlaylistItem();
        item.title = media.getTitle();
        item.media = MediaItem.from(media).toReference();
        item.quality = quality;

        if (media instanceof ShowOverview show) {
            item.thumb = show.getImages().getPoster();
        } else if (media instanceof MovieOverview movie) {
            item.thumb = movie.getImages().getPoster();
        } else if (media instanceof Episode episode) {
            item.thumb = episode.getThumb().orElse(null);
        }

        return item;
    }

    public static PlaylistItem fromMedia(Media media) {
        return fromMedia(media, null);
    }

    public static PlaylistItem fromMediaTrailer(MovieDetails media) {
        var item = fromMedia(media);
        item.url = media.getTrailer();
        return item;
    }
}
