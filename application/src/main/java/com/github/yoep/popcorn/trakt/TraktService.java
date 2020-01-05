package com.github.yoep.popcorn.trakt;

import com.github.yoep.popcorn.config.properties.PopcornProperties;
import lombok.RequiredArgsConstructor;
import org.springframework.security.oauth2.client.OAuth2RestOperations;
import org.springframework.stereotype.Service;
import org.springframework.web.util.UriComponentsBuilder;

@Service
@RequiredArgsConstructor
public class TraktService {
    private final OAuth2RestOperations traktTemplate;
    private final PopcornProperties popcornProperties;

    public void getWatched() {
        String url = UriComponentsBuilder.fromUri(popcornProperties.getTrakt().getUrl())
                .path("/sync/watched/")
                .toUriString();

        traktTemplate.getForEntity(url, String.class);
    }
}
