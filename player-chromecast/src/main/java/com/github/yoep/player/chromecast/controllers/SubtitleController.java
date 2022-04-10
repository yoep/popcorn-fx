package com.github.yoep.player.chromecast.controllers;

import com.github.yoep.player.chromecast.services.ChromecastService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.InputStreamResource;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;

@Slf4j
@Controller
@RequestMapping("/subtitle")
public record SubtitleController(ChromecastService chromecastService) {
    static final MediaType VTT_MEDIA_TYPE = new MediaType("text", "vtt");

    @GetMapping("{subtitle}")
    public ResponseEntity<InputStreamResource> retrieveSubtitle(@PathVariable("subtitle") String subtitle) {
        return chromecastService.retrieveVttSubtitle(subtitle)
                .map(InputStreamResource::new)
                .map(e -> ResponseEntity.ok()
                        .header(HttpHeaders.ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .header(HttpHeaders.ACCESS_CONTROL_ALLOW_METHODS, "GET,HEAD")
                        .header(HttpHeaders.ACCESS_CONTROL_ALLOW_HEADERS, HttpHeaders.CONTENT_TYPE)
                        .header(HttpHeaders.CONTENT_DISPOSITION, (String) null)
                        .contentType(VTT_MEDIA_TYPE)
                        .body(e))
                .orElse(ResponseEntity.notFound()
                        .build());
    }
}
