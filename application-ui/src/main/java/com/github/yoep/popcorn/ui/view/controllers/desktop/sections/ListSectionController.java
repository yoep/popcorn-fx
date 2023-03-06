package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.GenreChangeEvent;
import com.github.yoep.popcorn.ui.events.SortByChangeEvent;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.AbstractListSectionController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayMediaCardComponent;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
public class ListSectionController extends AbstractListSectionController implements Initializable {
    private final OverlayItemMetadataProvider metadataProvider = metadataProvider();
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final ImageService imageService;
    private final EventPublisher eventPublisher;

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
                                 ApplicationEventPublisher eventPublisher,
                                 ImageService imageService, EventPublisher eventPublisher1) {
        super(providerServices, viewLoader, localeText, eventPublisher);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.imageService = imageService;
        this.eventPublisher = eventPublisher1;
    }

    //endregion

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeScrollPane();
        initializeFailedPane();
        initializeOverlay();
        initializeFilter();
        initializeBackgroundImage();

        eventPublisher.register(CategoryChangedEvent.class, this::onCategoryChanged, EventPublisher.HIGHEST_ORDER);
        eventPublisher.register(GenreChangeEvent.class, this::onGenreChange);
        eventPublisher.register(SortByChangeEvent.class, this::onSortByChange);
    }

    private void initializeBackgroundImage() {
        imageService.loadResource("placeholder-background.jpg")
                        .thenAccept(e -> backgroundImage.setBackgroundImage(e));
    }

    private void initializeFilter() {
        var filter = viewLoader.load("components/filter.component.fxml");
        AnchorPane.setTopAnchor(filter, 5d);
        AnchorPane.setLeftAnchor(filter, 64d);
        AnchorPane.setRightAnchor(filter, 0d);
        listSection.getChildren().add(1, filter);
    }

    @Override
    protected Node createItemNode(Media item) {
        // load a new media card controller and inject it into the view
        var mediaCardComponent = new OverlayMediaCardComponent(item, localeText, imageService, metadataProvider, createItemListener());

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
            public void addListener(FavoriteEventCallback callback) {
                favoriteService.registerListener(callback);
            }

            @Override
            public void removeListener(FavoriteEventCallback callback) {
                favoriteService.removeListener(callback);
            }

            @Override
            public boolean isWatched(Media media) {
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
