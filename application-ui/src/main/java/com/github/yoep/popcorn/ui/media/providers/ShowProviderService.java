package com.github.yoep.popcorn.ui.media.providers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.activities.ErrorNotificationActivity;
import com.github.yoep.popcorn.ui.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.config.properties.ProviderProperties;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.models.Category;
import com.github.yoep.popcorn.ui.view.models.Genre;
import com.github.yoep.popcorn.ui.view.models.SortBy;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.text.MessageFormat;
import java.util.Arrays;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
public class ShowProviderService extends AbstractProviderService<Show> {
    private static final Category CATEGORY = Category.SERIES;

    private final ProviderProperties providerConfig;
    private final LocaleText localeText;

    public ShowProviderService(RestTemplate restTemplate,
                               ActivityManager activityManager,
                               PopcornProperties popcornConfig,
                               LocaleText localeText) {
        super(restTemplate, activityManager);
        this.providerConfig = popcornConfig.getProvider(CATEGORY.getProviderName());
        this.localeText = localeText;
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
    public void showDetails(Media media) {
        try {
            var show = getDetailsInternal(media.getId());
            activityManager.register((ShowSerieDetailsActivity) () -> show);
        } catch (Exception ex) {
            log.error("Failed to load show details, " + ex.getMessage(), ex);
            activityManager.register((ErrorNotificationActivity) () -> localeText.get(DetailsMessage.DETAILS_FAILED_TO_LOAD));
        }
    }

    public Page<Show> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        URI uri = getUriFor(getUri(), "shows", genre, sortBy, keywords, page);

        log.debug("Retrieving show provider page \"{}\"", uri);
        ResponseEntity<Show[]> shows = restTemplate.getForEntity(uri, Show[].class);

        return Optional.ofNullable(shows.getBody())
                .map(Arrays::asList)
                .map(PageImpl::new)
                .orElse(emptyPage());
    }

    private Show getDetailsInternal(String imdbId) {
        var uri = UriComponentsBuilder.fromUri(getUri())
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
    }

    private URI getUri() {
        //TODO: cycle through the uri's on failure
        return providerConfig.getUris().get(0);
    }
}
