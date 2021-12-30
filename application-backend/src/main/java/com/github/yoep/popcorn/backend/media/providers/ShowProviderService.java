package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.text.MessageFormat;
import java.util.Arrays;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
public class ShowProviderService extends AbstractProviderService<Show> {
    private static final Category CATEGORY = Category.SERIES;

    public ShowProviderService(RestTemplate restTemplate,
                               PopcornProperties popcornConfig,
                               SettingsService settingsService) {
        super(restTemplate);

        initializeUriProviders(settingsService.getSettings().getServerSettings(), popcornConfig.getProvider(CATEGORY.getProviderName()));
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<Show>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, StringUtils.EMPTY, page));
    }

    @Override
    public CompletableFuture<Page<Show>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    @Override
    public CompletableFuture<Show> getDetails(String imdbId) {
        return CompletableFuture.completedFuture(getDetailsInternal(imdbId));
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        try {
            return CompletableFuture.completedFuture(getDetailsInternal(media.getId()));
        } catch (Exception ex) {
            throw new MediaDetailsException(media, "Failed to load show details", ex);
        }
    }

    public Page<Show> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        return invokeWithUriProvider(apiUri -> {
            var uri = getUriFor(apiUri, "shows", genre, sortBy, keywords, page);

            log.debug("Retrieving show provider page \"{}\"", uri);
            ResponseEntity<Show[]> shows = restTemplate.getForEntity(uri, Show[].class);

            return Optional.ofNullable(shows.getBody())
                    .map(Arrays::asList)
                    .map(PageImpl::new)
                    .orElse(emptyPage());
        });
    }

    private Show getDetailsInternal(String imdbId) {
        return invokeWithUriProvider(apiUri -> {
            var uri = UriComponentsBuilder.fromUri(apiUri)
                    .path("show/{imdb_id}")
                    .build(imdbId);

            log.debug("Retrieving show details \"{}\"", uri);
            var response = restTemplate.getForEntity(uri, Show.class);

            if (response.getStatusCodeValue() < 200 || response.getStatusCodeValue() >= 300)
                throw new MediaException(
                        MessageFormat.format("Failed to retrieve the details of {0}, unexpected status code {1}", imdbId, response.getStatusCodeValue()));

            if (response.getBody() == null)
                throw new MediaException(MessageFormat.format("Failed to retrieve the details of {0}, response body is null", imdbId));

            return response.getBody();
        });
    }
}
