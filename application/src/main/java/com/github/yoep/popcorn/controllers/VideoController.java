package com.github.yoep.popcorn.controllers;

import com.github.yoep.popcorn.torrent.TorrentService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.ObjectUtils;
import org.springframework.core.io.FileSystemResource;
import org.springframework.core.io.support.ResourceRegion;
import org.springframework.http.*;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.io.IOException;

@Slf4j
@RestController
@RequestMapping("/video")
@RequiredArgsConstructor
public class VideoController {
    // set the default chunk size to 1 MB
    private static final long DEFAULT_CHUNK_SIZE = 10 * 1024 * 1024;

    private final TorrentService torrentService;

    @RequestMapping("/{filename}")
    public ResponseEntity<ResourceRegion> videoPart(@RequestHeader HttpHeaders headers,
                                                    HttpMethod method,
                                                    @PathVariable String filename) throws IOException {
        var torrentFile = torrentService.getTorrentFile(filename);

        // check if the torrent file exists
        if (!torrentFile.exists()) {
            log.warn("Torrent file \"{}\" does not exist, unable to serve video \"{}\"", torrentFile.getAbsolutePath(), filename);
            return ResponseEntity.notFound().build();
        }

        ResourceRegion region;
        var torrent = torrentService.getTorrent(filename);
        var video = new FileSystemResource(torrentFile);
        var videoLength = video.contentLength();
        var range = headers.getRange().stream().findFirst().orElse(null);
        var etag = Integer.toHexString((torrentFile.getAbsolutePath() + torrentFile.lastModified() + videoLength).hashCode());
        var defaultChunkSize = ObjectUtils.min(DEFAULT_CHUNK_SIZE, videoLength);

        if (range == null) {
            region = new ResourceRegion(video, 0, defaultChunkSize);
        } else {
            var start = range.getRangeStart(videoLength);

            // check if the requested start is larger than the file size
            // if so, return that the request cannot be fulfilled
            if (start > videoLength) {
                log.warn("Requested content range is invalid, start offset [{}] is larger than the file size [{}]", start, videoLength);
                return ResponseEntity.status(HttpStatus.REQUESTED_RANGE_NOT_SATISFIABLE)
                        .contentType(MediaType.TEXT_PLAIN)
                        .eTag(etag)
                        .build();
            }

            var end = range.getRangeEnd(videoLength);
            var chunkSize = ObjectUtils.min(defaultChunkSize, end);

            // check that the chunk size is not larger than the video size
            // if so, return only the remaining bytes
            if (start + chunkSize > videoLength) {
                chunkSize = videoLength - start;
            }

            // check if the chunk size contains any data
            // if not, return that the content has not been modified
            if (chunkSize == 0) {
                log.debug("Requested range contains [0] remaining bytes of the [{}] video file size", videoLength);
                return ResponseEntity.status(HttpStatus.NOT_MODIFIED)
                        .contentType(getContentType(video))
                        .eTag(etag)
                        .build();
            }

            region = new ResourceRegion(video, start, chunkSize);
        }

        // update the interested parts of the torrent
        torrent.setInterestedBytes(region.getPosition());

        log.trace("Serving video chunk \"{}-{}/{}\" for torrent stream \"{}\"", region.getPosition(), region.getCount(), videoLength, filename);
        return ResponseEntity.status(HttpStatus.PARTIAL_CONTENT)
                .header(HttpHeaders.ACCEPT_RANGES, "bytes")
                .contentType(getContentType(video))
                .eTag(etag)
                .body(region);
    }

    private MediaType getContentType(FileSystemResource video) {
        MediaType mediaType = MediaTypeFactory.getMediaType(video)
                .orElse(MediaType.APPLICATION_OCTET_STREAM);

        log.trace("Resolved video file \"{}\" as content type \"{}\"", video.getFilename(), mediaType);
        return mediaType;
    }
}
