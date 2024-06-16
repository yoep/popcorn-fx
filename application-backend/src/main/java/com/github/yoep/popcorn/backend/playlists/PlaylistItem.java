package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentFileInfoWrapper;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentInfoWrapper;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.Media;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.Optional;

@Data
@ToString(of = "title")
@EqualsAndHashCode(exclude = {"parentMedia", "media"}, callSuper = false)
@NoArgsConstructor
@Structure.FieldOrder({"url", "title", "caption", "thumb", "quality", "parentMedia", "media", "autoResumeTimestamp", "subtitlesEnabled", "subtitleInfo", "torrentInfo", "torrentFileInfo"})
public class PlaylistItem extends Structure implements Closeable {
    public static class ByReference extends PlaylistItem implements Structure.ByReference {
    }

    public String url;
    public String title;
    public String caption;
    public String thumb;
    public String quality;
    public MediaItem.ByReference parentMedia;
    public MediaItem.ByReference media;
    public Long autoResumeTimestamp;
    public byte subtitlesEnabled;
    public SubtitleInfo.ByReference subtitleInfo;
    public TorrentInfoWrapper.ByReference torrentInfo;
    public TorrentFileInfoWrapper.ByReference torrentFileInfo;

    public PlaylistItem(String url, String title) {
        this.url = url;
        this.title = title;
    }

    public PlaylistItem(String url, String title, String thumb, MediaItem.ByReference media) {
        this.url = url;
        this.title = title;
        this.thumb = thumb;
        this.media = media;
    }

    @Builder
    public PlaylistItem(String url,
                        String title,
                        String caption,
                        String thumb,
                        String quality,
                        MediaItem.ByReference parentMedia,
                        MediaItem.ByReference media,
                        Long autoResumeTimestamp,
                        boolean subtitlesEnabled,
                        SubtitleInfo.ByReference subtitleInfo,
                        TorrentInfoWrapper.ByReference torrentInfo,
                        TorrentFileInfoWrapper.ByReference torrentFileInfo) {
        this.url = url;
        this.title = title;
        this.caption = caption;
        this.thumb = thumb;
        this.quality = quality;
        this.parentMedia = parentMedia;
        this.media = media;
        this.autoResumeTimestamp = autoResumeTimestamp;
        this.subtitlesEnabled = (byte) (subtitlesEnabled ? 1 : 0);
        this.subtitleInfo = subtitleInfo;
        this.torrentInfo = torrentInfo;
        this.torrentFileInfo = torrentFileInfo;
    }

    public Optional<String> getUrl() {
        return Optional.ofNullable(url);
    }

    public Optional<String> getCaption() {
        return Optional.ofNullable(caption);
    }

    public Optional<String> getThumb() {
        return Optional.ofNullable(thumb);
    }

    public Optional<Media> getParentMedia() {
        return Optional.ofNullable(parentMedia)
                .map(MediaItem::getMedia);
    }

    public Optional<Media> getMedia() {
        return Optional.ofNullable(media)
                .map(MediaItem::getMedia);
    }

    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }

    public Optional<Long> getAutoResumeTimestamp() {
        return Optional.ofNullable(autoResumeTimestamp);
    }

    public void setSubtitlesEnabled(boolean subtitlesEnabled) {
        this.subtitlesEnabled = (byte) (subtitlesEnabled ? 1 : 0);
    }

    public boolean isSubtitlesEnabled() {
        return subtitlesEnabled == 1;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    public static PlaylistItem fromMediaTrailer(MovieDetails media) {
        var item = new PlaylistItem();
        item.url = media.getTrailer();
        item.title = media.getTitle();
        item.caption = "Trailer";
        item.thumb = media.getImages().getPoster();
        item.setSubtitlesEnabled(false);
        return item;
    }
}
