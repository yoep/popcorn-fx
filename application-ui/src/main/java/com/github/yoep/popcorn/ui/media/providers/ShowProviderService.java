package com.github.yoep.popcorn.ui.media.providers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.ui.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.models.Category;
import com.github.yoep.popcorn.ui.view.models.Genre;
import com.github.yoep.popcorn.ui.view.models.SortBy;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.context.ApplicationEventPublisher;
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

    private final LocaleText localeText;

    public ShowProviderService(RestTemplate restTemplate,
                               ApplicationEventPublisher eventPublisher,
                               PopcornProperties popcornConfig,
                               LocaleText localeText,
                               SettingsService settingsService) {
        super(restTemplate, eventPublisher);
        this.localeText = localeText;

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
    public CompletableFuture<Boolean> showDetails(Media media) {
        try {
            var show = getDetailsInternal(media.getId());
            eventPublisher.publishEvent(new ShowSerieDetailsEvent(this, show));

            return CompletableFuture.completedFuture(true);
        } catch (Exception ex) {
            log.error("Failed to load show details, " + ex.getMessage(), ex);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(DetailsMessage.DETAILS_FAILED_TO_LOAD)));
        }

        return CompletableFuture.completedFuture(false);
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
