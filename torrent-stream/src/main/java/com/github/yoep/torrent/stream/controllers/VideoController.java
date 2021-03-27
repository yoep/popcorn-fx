package com.github.yoep.torrent.stream.controllers;

import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.torrent.stream.services.MediaService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.support.ResourceRegion;
import org.springframework.http.HttpHeaders;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.io.IOException;

@Slf4j
@RestController
@RequestMapping("/video")
@RequiredArgsConstructor
public class VideoController {
    private static final String JAVA_MEDIA_USER_AGENT = "Java";
    private static final String HEADER_DLNA_TRANSFER_MODE = "TransferMode.dlna.org";
    private static final String ACCEPT_RANGES_TYPE = "bytes";
    private static final String CONNECTION_TYPE = "keep-alive";
    private static final String DLNA_TRANSFER_MODE_TYPE = "Streaming";

    private final TorrentStreamService streamService;
    private final MediaService mediaService;

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
        long rangeStart;
        long rangeEnd;
        var video = torrent.get().stream();
        var videoLength = video.contentLength();
        var range = headers.getRange().stream().findFirst().orElse(null);
        var agent = headers.getFirst(HttpHeaders.USER_AGENT);
        var status = HttpStatus.PARTIAL_CONTENT;

        // check if the range header is present
        // if so, use the range header to determine the chunk
        if (range != null) {
            rangeStart = range.getRangeStart(videoLength);
            rangeEnd = range.getRangeEnd(videoLength);

            // check that the chunk size is not larger than the video size
            // if so, return only the remaining bytes
            if (rangeStart + rangeEnd > videoLength) {
                rangeEnd = videoLength - rangeStart;
            }
        } else {
            rangeStart = 0;
            rangeEnd = videoLength;
        }

        // check if the requested start is larger than the file size
        // if so, return that the request cannot be fulfilled
        if (rangeStart > videoLength) {
            log.warn("Requested content range is invalid, start offset [{}] is larger than the file size [{}]", rangeStart, videoLength);
            return ResponseEntity.status(HttpStatus.REQUESTED_RANGE_NOT_SATISFIABLE)
                    .contentType(MediaType.TEXT_PLAIN)
                    .build();
        }

        // create the resource region based on the range start and end size
        var region = new ResourceRegion(video, rangeStart, rangeEnd);

        // check if the agent is Java media
        // if so, return OK as status as the media player doesn't accept any other status as success
        if (agent != null && agent.contains(JAVA_MEDIA_USER_AGENT)) {
            status = HttpStatus.OK;
        }

        log.trace("Serving video chunk \"{}-{}/{}\" for torrent stream \"{}\"",
                region.getPosition(), region.getCount(), videoLength, filename);
        var contentType = mediaService.contentType(video);
        var response = ResponseEntity.status(status)
                .header(HttpHeaders.ACCEPT_RANGES, ACCEPT_RANGES_TYPE)
                .header(HttpHeaders.CONNECTION, CONNECTION_TYPE)
                .header(HEADER_DLNA_TRANSFER_MODE, DLNA_TRANSFER_MODE_TYPE)
                .contentType(contentType)
                .body(region);
        log.trace("Responding to video request \"{}\" with status {} and headers {}", filename, response.getStatusCodeValue(), response.getHeaders());

        return response;
    }


}
