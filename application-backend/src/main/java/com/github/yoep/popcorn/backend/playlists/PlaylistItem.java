package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;
import lombok.ToString;

import java.io.Closeable;
import java.io.IOException;
import java.util.Optional;

@Data
@ToString
@EqualsAndHashCode(callSuper = false)
@NoArgsConstructor
@Structure.FieldOrder({"url", "title", "thumb", "media"})
public class PlaylistItem extends Structure implements Closeable {
    public static class ByReference extends PlaylistItem implements Structure.ByReference {
    }

    public String url;
    public String title;
    public String thumb;
    public MediaItem.ByReference media;

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

    @Override
    public void close() throws IOException {
        setAutoSynch(false);
    }

    public static PlaylistItem fromMedia(Media media) {
        var item = new PlaylistItem();
        item.title = media.getTitle();
        item.media = MediaItem.from(media).toReference();

        if (media instanceof ShowOverview show) {
            item.thumb = show.getImages().getPoster();
        } else if (media instanceof MovieOverview movie) {
            item.thumb = movie.getImages().getPoster();
        } else if (media instanceof Episode episode) {
            item.thumb = episode.getThumb().orElse(null);
        }

        return item;
    }
}
