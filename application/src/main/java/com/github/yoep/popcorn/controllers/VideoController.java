package com.github.yoep.popcorn.controllers;

import com.github.yoep.popcorn.torrent.TorrentService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.ObjectUtils;
import org.springframework.core.io.FileSystemResource;
import org.springframework.core.io.support.ResourceRegion;
import org.springframework.http.*;
import org.springframework.web.bind.annotation.*;

import java.io.IOException;

@Slf4j
@RestController
@RequestMapping("/video")
@RequiredArgsConstructor
public class VideoController {
    // set the default chunk size to 2 MB
    private static final long DEFAULT_CHUNK_SIZE = 2 * 1024 * 1024;

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
            var end = range.getRangeEnd(videoLength);
            var chunkSize = ObjectUtils.min(defaultChunkSize, end);

            region = new ResourceRegion(video, start, chunkSize);
        }

        // update the interested parts of the torrent
        torrent.setInterestedBytes(region.getPosition());

        log.debug("Serving video chunk \"{}-{}/{}\" for torrent stream \"{}\"", region.getPosition(), region.getCount(), videoLength, filename);
        return ResponseEntity.status(HttpStatus.PARTIAL_CONTENT)
                .header(HttpHeaders.ACCEPT_RANGES, "bytes")
                .contentType(MediaTypeFactory.getMediaType(video)
                        .orElse(MediaType.APPLICATION_OCTET_STREAM))
                .eTag(etag)
                .body(region);
    }
}
