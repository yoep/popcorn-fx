package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FavoriteEvent;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.MediaParsingException;
import com.github.yoep.popcorn.backend.media.providers.MediaRetrievalException;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SearchEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.messages.ListMessage;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controllers.common.components.MediaCardComponent;
import com.github.yoep.popcorn.ui.view.controllers.common.components.TvMediaCardComponent;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.controls.InfiniteScrollItemFactory;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class ListSectionController extends AbstractListSectionController implements Initializable {
    static final double LEFT_SPACING_DESKTOP = 64.0;
    static final double LEFT_SPACING_TV = 128.0;

    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final ImageService imageService;
    private final ApplicationConfig applicationConfig;

    private final OverlayItemMetadataProvider metadataProvider = metadataProvider();
    private final OverlayItemListener overlayItemListener = createItemListener();

    @FXML
    AnchorPane listSection;
    @FXML
    BackgroundImageCover backgroundImage;

    //region Constructors

    @Builder
    public ListSectionController(List<ProviderService<? extends Media>> providerServices,
                                 FavoriteService favoriteService,
                                 WatchedService watchedService,
                                 ViewLoader viewLoader,
                                 LocaleText localeText,
                                 EventPublisher eventPublisher,
                                 ImageService imageService,
                                 ApplicationConfig applicationConfig) {
        super(providerServices, viewLoader, localeText, eventPublisher);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.imageService = imageService;
        this.applicationConfig = applicationConfig;
    }

    //endregion

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeScrollPane();
        initializeFailedPane();
        initializeOverlay();
        initializeBackgroundImage();
        initializeFilter();

        eventPublisher.register(CategoryChangedEvent.class, this::onCategoryChanged, EventPublisher.HIGHEST_ORDER);
        eventPublisher.register(GenreChangeEvent.class, this::onGenreChange);
        eventPublisher.register(SortByChangeEvent.class, this::onSortByChange);
        eventPublisher.register(SearchEvent.class, this::onSearch);
    }

    private void initializeScrollPane() {
        AnchorPane.setTopAnchor(scrollPane, applicationConfig.isTvMode() ? 50.0 : 35.0);
        AnchorPane.setLeftAnchor(scrollPane, applicationConfig.isTvMode() ? LEFT_SPACING_TV : LEFT_SPACING_DESKTOP);
        scrollPane.setLoaderFactory(() -> viewLoader.load("common/components/loading-card.component.fxml"));
        scrollPane.setItemFactory(new InfiniteScrollItemFactory<>() {
            @Override
            public CompletableFuture<List<Media>> loadPage(int page) {
                return loadItems(page)
                        .exceptionally(throwable -> onMediaRequestFailed(throwable))
                        .thenApply(Arrays::asList);
            }

            @Override
            public Node createCell(Media item) {
                return createItemNode(item);
            }
        });
        scrollPane.requestFocus();
    }

    private void initializeBackgroundImage() {
        imageService.loadResource("placeholder-background.jpg")
                .thenAccept(e -> backgroundImage.setBackgroundImage(e));
    }

    private void initializeFilter() {
        var filter = viewLoader.load("components/filter.component.fxml");

        if (applicationConfig.isTvMode()) {
            AnchorPane.setTopAnchor(filter, 0d);
            AnchorPane.setBottomAnchor(filter, 0d);
            AnchorPane.setLeftAnchor(filter, LEFT_SPACING_TV);
        } else {
            AnchorPane.setTopAnchor(filter, 5d);
            AnchorPane.setRightAnchor(filter, 0d);
            AnchorPane.setLeftAnchor(filter, LEFT_SPACING_DESKTOP);
        }

        listSection.getChildren().add(applicationConfig.isTvMode() ? 2 : 1, filter);
    }

    @Override
    protected Node createItemNode(Media item) {
        TvMediaCardComponent mediaCardComponent;

        if (applicationConfig.isTvMode()) {
            mediaCardComponent = new TvMediaCardComponent(item, imageService, metadataProvider, overlayItemListener);
        } else {
            // load a new media card controller and inject it into the view
            mediaCardComponent = new MediaCardComponent(item, localeText, imageService, metadataProvider, overlayItemListener);
        }

        return viewLoader.load("components/media-card-overlay.component.fxml", mediaCardComponent);
    }

    private void onRetryMediaLoading() {
        providerServices.forEach(ProviderService::resetApiAvailability);
        scrollPane.reset();
        scrollPane.loadNewPage();
    }

    private CategoryChangedEvent onCategoryChanged(CategoryChangedEvent event) {
        this.category = event.getCategory();
        // reset the genre & sort by as they might be different in the new category
        // these will be automatically filled in again as the category change also triggers a GenreChangeActivity & SortByChangeActivity
        this.genre = null;
        this.sortBy = null;

        reset();
        invokeNewPageLoad();
        return event;
    }

    private GenreChangeEvent onGenreChange(GenreChangeEvent event) {
        this.genre = event.getGenre();

        reset();
        invokeNewPageLoad();
        return event;
    }

    private SortByChangeEvent onSortByChange(SortByChangeEvent event) {
        this.sortBy = event.getSortBy();

        reset();
        invokeNewPageLoad();
        return event;
    }

    private SearchEvent onSearch(SearchEvent event) {
        var newValue = event.getValue().orElse(null);

        if (Objects.equals(search, newValue))
            return event;

        this.search = newValue;
        reset();
        invokeNewPageLoad();
        return event;
    }

    private OverlayItemListener createItemListener() {
        return new OverlayItemListener() {
            @Override
            public void onClicked(Media media) {
                onItemClicked(media);
            }

            @Override
            public void onFavoriteChanged(Media media, boolean newValue) {
                if (newValue) {
                    favoriteService.addToFavorites(media);
                } else {
                    favoriteService.removeFromFavorites(media);
                }
            }

            @Override
            public void onWatchedChanged(Media media, boolean newValue) {
                if (newValue) {
                    watchedService.addToWatchList(media);
                } else {
                    watchedService.removeFromWatchList(media);
                }
            }
        };
    }

    private void releaseCurrentLoadRequest() {
        if (!this.currentLoadRequest.isDone()) {
            log.warn("Unable to release the current load request, load request is still in progress");
            return;
        }

        this.currentLoadRequest = null;
    }

    private Media[] onMediaRequestFailed(Throwable throwable) {
        releaseCurrentLoadRequest();

        // check if the media request was cancelled
        // if so, ignore this failure
        if (throwable instanceof CancellationException || throwable.getCause() instanceof CancellationException) {
            log.trace("Media request has been cancelled by the user");
            return new Media[0];
        }

        var rootCause = throwable.getCause();
        var message = new AtomicReference<>(localeText.get(ListMessage.GENERIC));
        log.error("Failed to retrieve media list, " + rootCause.getMessage(), throwable);

        // verify if the parsing of the page failed
        // if so, ignore this page and load the next one
        if (rootCause instanceof MediaParsingException) {
            if (numberOfPageFailures < 2) {
                log.warn("Media page {} has been skipped due to a parsing error, loading next page", scrollPane.getPage());
                numberOfPageFailures++;
                this.eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(DetailsMessage.DETAILS_INVALID_RESPONSE_RECEIVED)));
                // force load the next page as this method is currently in the updating state
                // of the scroll pane, calling the normal load won't do anything
                scrollPane.forceLoadNewPage();
                return new Media[0];
            }
        }

        // verify if an invalid response was received from the backend
        // if so, show the status code
        if (rootCause instanceof MediaRetrievalException) {
            message.set(localeText.get(ListMessage.API_UNAVAILABLE, 500));
        }

        Platform.runLater(() -> {
            failedText.setText(message.get());
            failedPane.setVisible(true);
            hideOverlay();
        });

        this.eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(ListMessage.RETRIEVAL_FAILED)));
        return new Media[0];
    }

    private void hideOverlay() {
        overlay.setVisible(false);

        overlay.getChildren().clear();
        loadingIndicator = null;
    }

    private CompletableFuture<Media[]> retrieveMediaPage(ProviderService<? extends Media> provider, int page) {
        if (search == null || search.isBlank()) {
            currentLoadRequest = provider.getPage(genre, sortBy, page);
        } else {
            currentLoadRequest = provider.getPage(genre, sortBy, page, search);
        }

        return currentLoadRequest
                .thenApply(this::onMediaRequestCompleted)
                .exceptionally(this::onMediaRequestFailed);
    }

    private Media[] onMediaRequestCompleted(final List<? extends Media> page) {
        releaseCurrentLoadRequest();

        // filter out any duplicate items
        return page.stream()
                .filter(e -> !scrollPane.contains(e))
                .toArray(Media[]::new);
    }

    private void handleDetailsResponse(Media media, Throwable throwable) {
        if (throwable == null) {
            Platform.runLater(this::hideOverlay);

            if (media instanceof MovieDetails) {
                this.eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, (MovieDetails) media));
            } else if (media instanceof ShowDetails) {
                this.eventPublisher.publishEvent(new ShowSerieDetailsEvent(this, (ShowDetails) media));
            }
        } else {
            log.error(throwable.getMessage(), throwable);
            this.eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(DetailsMessage.DETAILS_FAILED_TO_LOAD)));
            Platform.runLater(this::hideOverlay);
        }
    }

    private void showMediaDetails(Media media, ProviderService<? extends Media> provider) {
        provider.retrieveDetails(media)
                .whenComplete(this::handleDetailsResponse);
    }

    private void onItemClicked(Media media) {
        showOverlay();
        providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .ifPresent(provider -> showMediaDetails(media, provider));
    }

    private CompletableFuture<Media[]> loadItems(final int page) {
        // hide the failed pane in case it might be visible from the last failure
        Platform.runLater(() -> failedPane.setVisible(false));

        // cancel the current load request if one is present
        // and has not yet been completed
        if (currentLoadRequest != null && !currentLoadRequest.isDone())
            currentLoadRequest.cancel(true);

        log.trace("Retrieving media page {} for {} category", page, category);
        var provider = providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst();

        if (provider.isPresent()) {
            return retrieveMediaPage(provider.get(), page);
        } else {
            log.error("No provider service found for \"{}\" category", category);
            return CompletableFuture.completedFuture(new Media[0]);
        }
    }

    @FXML
    void onRetryListLoading(MouseEvent event) {
        event.consume();
        onRetryMediaLoading();
    }

    private OverlayItemMetadataProvider metadataProvider() {
        return new OverlayItemMetadataProvider() {
            @Override
            public boolean isLiked(Media media) {
                return favoriteService.isLiked(media);
            }

            @Override
            public void addListener(FxCallback<FavoriteEvent> callback) {
                favoriteService.registerListener(callback);
            }

            @Override
            public void removeListener(FxCallback<FavoriteEvent> callback) {
                favoriteService.removeListener(callback);
            }

            @Override
            public CompletableFuture<Boolean> isWatched(Media media) {
                return watchedService.isWatched(media);
            }

            @Override
            public void addListener(WatchedEventCallback callback) {
                watchedService.registerListener(callback);
            }

            @Override
            public void removeListener(WatchedEventCallback callback) {
                watchedService.removeListener(callback);
            }
        };
    }
}
