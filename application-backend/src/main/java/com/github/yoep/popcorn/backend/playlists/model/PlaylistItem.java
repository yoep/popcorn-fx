package com.github.yoep.popcorn.backend.playlists.model;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.Media;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import lombok.Builder;

import java.util.Optional;

@Builder
public record PlaylistItem(String url,
                           String title,
                           String caption,
                           String thumb,
                           String quality,
                           MediaItem.ByReference parentMedia,
                           MediaItem.ByReference media,
                           Long autoResumeTimestamp,
                           boolean subtitlesEnabled,
                           SubtitleInfo subtitleInfo,
                           TorrentInfo torrentInfo,
                           TorrentFileInfo torrentFileInfo) {

    public Optional<String> getUrl() {
        return Optional.ofNullable(url);
    }

    public Optional<String> getCaption() {
        return Optional.ofNullable(caption);
    }

    public Optional<String> getThumb() {
        return Optional.ofNullable(thumb);
    }

    public Optional<Media> getMedia() {
        return Optional.ofNullable(media)
                .map(MediaItem::getMedia);
    }

    public Optional<String> getQuality() {
        return Optional.ofNullable(quality);
    }

    public static PlaylistItem from(com.github.yoep.popcorn.backend.playlists.ffi.PlaylistItem item) {
        return PlaylistItem.builder()
                .url(item.url)
                .title(item.title)
                .caption(item.caption)
                .thumb(item.thumb)
                .quality(item.quality)
                .parentMedia(item.parentMedia)
                .media(item.media)
                .autoResumeTimestamp(item.autoResumeTimestamp)
                .subtitlesEnabled(item.subtitlesEnabled != 0)
                .subtitleInfo(Optional.ofNullable(item.subtitleInfo)
                        .map(SubtitleInfo::from)
                        .orElse(null))
                .torrentInfo(item.torrentInfo)
                .torrentFileInfo(item.torrentFileInfo)
                .build();
    }
}
