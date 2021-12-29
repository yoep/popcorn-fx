package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.AbstractListSectionController;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.SimpleMediaCardComponent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

import java.util.List;

@Slf4j
public class ListSectionController extends AbstractListSectionController implements Initializable {
    private final WatchedService watchedService;
    private final ImageService imageService;

    private boolean listHasBeenReset;

    //region Constructors

    public ListSectionController(List<ProviderService<? extends Media>> providerServices,
                                 ViewLoader viewLoader,
                                 LocaleText localeText,
                                 WatchedService watchedService,
                                 ImageService imageService) {
        super(providerServices, viewLoader, localeText);
        this.watchedService = watchedService;
        this.imageService = imageService;
    }

    //endregion

    //region Methods

    @EventListener(PlayMediaEvent.class)
    public void onPlayerMedia() {
        // release all items from the scroll list
        // this should free some memory used by the images in the scroll list
        // and we assume that when we're in TV mode,
        // we don't have much memory and want to allocate the memory to the video playback
        reset();
        listHasBeenReset = true;

        // request the JVM to execute a garbage collection
        System.gc();
    }

    @EventListener(CloseDetailsEvent.class)
    public void onDetailsClosed() {
        // if the items have been removed from the list
        // it will be empty when the details are being closed
        // to resolve this, request the list to load a new page
        if (listHasBeenReset) {
            invokeNewPageLoad();
            listHasBeenReset = false;
        }
    }

    //endregion

    //region Functions

    @Override
    protected Node creatItemNode(Media item) {
        item.setWatched(watchedService.isWatched(item));

        var mediaCardComponent = new SimpleMediaCardComponent(item, localeText, imageService, this::onItemClicked);

        return viewLoader.load("components/media-card-simple.component.fxml", mediaCardComponent);
    }

    //endregion
}
