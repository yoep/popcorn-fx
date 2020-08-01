package com.github.yoep.torrent.stream.controllers;

import com.github.yoep.torrent.adapter.TorrentStreamService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;
import org.springframework.core.io.support.ResourceRegion;
import org.springframework.http.*;
import org.springframework.web.bind.annotation.*;

import java.io.IOException;

@Slf4j
@RestController
@RequestMapping("/video")
@RequiredArgsConstructor
public class VideoController {
    private static final String JAVA_MEDIA_USER_AGENT = "Java";

    private final TorrentStreamService streamService;

    @RequestMapping(value = "/{filename}", method = RequestMethod.GET)
    public ResponseEntity<ResourceRegion> videoPart(@RequestHeader HttpHeaders headers,
                                                    @PathVariable String filename) throws IOException {
        var torrent = streamService.resolve(filename);

        // check if the torrent exists
        if (torrent.isEmpty()) {
            log.warn("Torrent \"{}\" does not exist, unable to serve video", filename);
            return ResponseEntity.notFound().build();
        }

        log.trace("Received request headers {} for video {}", headers, filename);
        ResourceRegion region;
        var video = torrent.get().stream();
        var videoLength = video.contentLength();
        var range = headers.getRange().stream().findFirst().orElse(null);
        var agent = headers.getFirst(HttpHeaders.USER_AGENT);
        var status = HttpStatus.PARTIAL_CONTENT;

        if (range == null) {
            region = new ResourceRegion(video, 0, videoLength);
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

            var chunkSize = range.getRangeEnd(videoLength);

            // check that the chunk size is not larger than the video size
            // if so, return only the remaining bytes
            if (start + chunkSize > videoLength) {
                chunkSize = videoLength - start;
            }

            region = new ResourceRegion(video, start, chunkSize);
        }

        // check if the agent is Java media
        // if so, return OK as status as the media player doesn't accept any other status as success
        if (agent != null && agent.contains(JAVA_MEDIA_USER_AGENT)) {
            status = HttpStatus.OK;
        }

        log.trace("Serving video chunk \"{}-{}/{}\" for torrent stream \"{}\"",
                region.getPosition(), region.getCount(), videoLength, filename);
        ResponseEntity<ResourceRegion> response = ResponseEntity.status(status)
                .header(HttpHeaders.ACCEPT_RANGES, "bytes")
                .header(HttpHeaders.CONNECTION, "keep-alive")
                .header("TransferMode.dlna.org", "Streaming")
                .contentType(getContentType(video))
                .body(region);
        log.trace("Responding to video request \"{}\" with status {} and headers {}", filename, response.getStatusCodeValue(), response.getHeaders());

        return response;
    }

    private MediaType getContentType(Resource video) {
        var mediaType = MediaTypeFactory.getMediaType(video)
                .orElse(MediaType.APPLICATION_OCTET_STREAM);

        log.trace("Resolved video file \"{}\" as content type \"{}\"", video.getFilename(), mediaType);
        return mediaType;
    }
}
