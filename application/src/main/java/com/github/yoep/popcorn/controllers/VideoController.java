package com.github.yoep.popcorn.controllers;

import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.config.properties.StreamingProperties;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.models.Torrent;
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
    private final PopcornProperties properties;
    private final TorrentService torrentService;

    @RequestMapping(value = "/{filename}", method = RequestMethod.GET)
    public ResponseEntity<ResourceRegion> videoPart(@RequestHeader HttpHeaders headers,
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
        var defaultChunkSize = ObjectUtils.min(streamingProperties().getChunkSize(), videoLength);

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
                        .build();
            }

            var end = range.getRangeEnd(videoLength);
            var chunkSize = ObjectUtils.min(defaultChunkSize, end);

            // check that the chunk size is not larger than the video size
            // if so, return only the remaining bytes
            if (start + chunkSize > videoLength) {
                chunkSize = videoLength - start;
            }

            region = new ResourceRegion(video, start, chunkSize);
        }

        // request the torrent to prioritize the requested bytes
        updateTorrentPriorityAndWait(torrent, region);

        log.trace("Serving video chunk \"{}-{}/{}\" for torrent stream \"{}\"",
                region.getPosition(), region.getCount(), videoLength, filename);
        return ResponseEntity.status(HttpStatus.PARTIAL_CONTENT)
                .header(HttpHeaders.ACCEPT_RANGES, "bytes")
                .contentType(getContentType(video))
                .body(region);
    }

    private void updateTorrentPriorityAndWait(Torrent torrent, ResourceRegion region) {
        // update the interested parts of the torrent
        torrent.setInterestedBytes(region.getPosition());

        // block the response until the requested parts are present
        //        while (!torrent.hasBytes(region.getPosition())) {
        //            // do nothing and wait for the torrent to download them
        //        }
    }

    private MediaType getContentType(FileSystemResource video) {
        MediaType mediaType = MediaTypeFactory.getMediaType(video)
                .orElse(MediaType.APPLICATION_OCTET_STREAM);

        log.trace("Resolved video file \"{}\" as content type \"{}\"", video.getFilename(), mediaType);
        return mediaType;
    }

    private StreamingProperties streamingProperties() {
        return properties.getStreaming();
    }
}
