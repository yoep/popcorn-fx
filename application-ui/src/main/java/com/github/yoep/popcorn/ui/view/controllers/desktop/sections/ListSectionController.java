package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.AbstractListSectionController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayMediaCardComponent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.input.MouseEvent;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;

import java.util.List;

@Slf4j
public class ListSectionController extends AbstractListSectionController implements Initializable {
    private final OverlayItemMetadataProvider metadataProvider = metadataProvider();
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final ImageService imageService;

    //region Constructors

    @Builder
    public ListSectionController(List<ProviderService<? extends Media>> providerServices,
                                 FavoriteService favoriteService,
                                 WatchedService watchedService,
                                 ViewLoader viewLoader,
                                 LocaleText localeText,
                                 ApplicationEventPublisher eventPublisher,
                                 ImageService imageService) {
        super(providerServices, viewLoader, localeText, eventPublisher);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.imageService = imageService;
    }

    //endregion

    //region Functions

    @Override
    protected Node creatItemNode(Media item) {
        // update the watched & liked states of the media item with the latest information
        item.setWatched(watchedService.isWatched(item));

        // load a new media card controller and inject it into the view
        var mediaCardComponent = new OverlayMediaCardComponent(item, localeText, imageService, metadataProvider, createItemListener());

        return viewLoader.load("components/media-card-overlay.component.fxml", mediaCardComponent);
    }

    private void onRetryMediaLoading() {
        providerServices.forEach(ProviderService::resetApiAvailability);
        scrollPane.reset();
        scrollPane.loadNewPage();
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
        };
    }

    //endregion
}
