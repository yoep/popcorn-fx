package com.github.yoep.popcorn.view.controllers.tv.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.media.providers.ProviderService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.watched.WatchedService;
import com.github.yoep.popcorn.view.controllers.common.sections.AbstractListSectionController;
import com.github.yoep.popcorn.view.controllers.tv.components.SimpleMediaCardComponent;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
public class ListSectionController extends AbstractListSectionController implements Initializable {
    private final WatchedService watchedService;
    private final ImageService imageService;

    private boolean requestFocus;

    //region Constructors

    public ListSectionController(ActivityManager activityManager,
                                 List<ProviderService<? extends Media>> providerServices,
                                 ViewLoader viewLoader,
                                 LocaleText localeText,
                                 WatchedService watchedService,
                                 ImageService imageService) {
        super(activityManager, providerServices, viewLoader, localeText);
        this.watchedService = watchedService;
        this.imageService = imageService;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializePageListener();
    }

    private void initializePageListener() {
        scrollPane.pageProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue.intValue() == 1) {
                requestFocus = true;
            }
        });
    }

    //endregion

    //region Functions

    @Override
    protected Node creatItemNode(Media item) {
        item.setWatched(watchedService.isWatched(item));

        var mediaCardComponent = new SimpleMediaCardComponent(item, localeText, imageService,  this::onItemClicked);

        // check if this media card item should request the focus
        // update the request focus later on back to false so only one item requests the focus
        if (requestFocus) {
            mediaCardComponent.setRequestFocus(true);
            requestFocus = false;
        }

        return viewLoader.load("components/media-card-simple.component.fxml", mediaCardComponent);
    }

    private void onItemClicked(Media media) {
        providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .ifPresent(provider -> provider.showDetails(media));
    }

    //endregion
}
