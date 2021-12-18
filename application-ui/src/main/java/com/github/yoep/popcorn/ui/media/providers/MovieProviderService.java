package com.github.yoep.popcorn.ui.media.providers;

import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.models.Category;
import com.github.yoep.popcorn.ui.view.models.Genre;
import com.github.yoep.popcorn.ui.view.models.SortBy;
import lombok.extern.slf4j.Slf4j;
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
public class MovieProviderService extends AbstractProviderService<Movie> {
    private static final Category CATEGORY = Category.MOVIES;

    public MovieProviderService(RestTemplate restTemplate,
                                ApplicationEventPublisher eventPublisher,
                                PopcornProperties popcornConfig,
                                SettingsService settingsService) {
        super(restTemplate, eventPublisher);

        initializeUriProviders(settingsService.getSettings().getServerSettings(), popcornConfig.getProvider(CATEGORY.getProviderName()));
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<Movie>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, "", page));
    }

    @Override
    public CompletableFuture<Page<Movie>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    @Override
    public CompletableFuture<Movie> getDetails(String imdbId) {
        return invokeWithUriProvider(apiUri -> {
            var uri = UriComponentsBuilder.fromUri(apiUri)
                    .path("/movie/{id}")
                    .build(imdbId);

            log.debug("Retrieving movie details \"{}\"", uri);
            var response = restTemplate.getForEntity(uri, Movie.class);

            if (response.getBody() == null) {
                return CompletableFuture.failedFuture(new MediaException(MessageFormat.format("Failed to retrieve the details of {0}", imdbId)));
            }

            return CompletableFuture.completedFuture(response.getBody());
        });
    }

    @Override
    public CompletableFuture<Boolean> showDetails(Media media) {
        final Movie movie = (Movie) media;

        eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, movie));
        return CompletableFuture.completedFuture(true);
    }

    public Page<Movie> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        return invokeWithUriProvider(apiUri -> {
            var uri = getUriFor(apiUri, "movies", genre, sortBy, keywords, page);

            log.debug("Retrieving movie provider page \"{}\"", uri);
            ResponseEntity<Movie[]> items = restTemplate.getForEntity(uri, Movie[].class);

            return Optional.ofNullable(items.getBody())
                    .map(Arrays::asList)
                    .map(PageImpl::new)
                    .orElse(emptyPage());
        });
    }
}
