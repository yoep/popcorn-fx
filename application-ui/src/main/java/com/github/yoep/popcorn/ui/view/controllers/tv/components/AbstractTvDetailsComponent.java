package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.messages.MediaMessage;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractDetailsComponent;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.context.ApplicationEventPublisher;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;

@Slf4j
public abstract class AbstractTvDetailsComponent<T extends Media> extends AbstractDetailsComponent<T> implements Initializable {
    protected final ApplicationEventPublisher eventPublisher;
    protected final SubtitleService subtitleService;
    protected final WatchedService watchedService;

    @FXML
    protected Overlay overlay;
    @FXML
    protected Pane subtitleButton;
    @FXML
    protected Label subtitleLabel;
    @FXML
    protected Pane qualityButton;
    @FXML
    protected Label qualityButtonLabel;
    @FXML
    protected Label rating;

    protected ListView<String> qualityList;
    protected ListView<SubtitleInfo> subtitleList;
    protected String quality;
    protected SubtitleInfo subtitle;

    private CompletableFuture<List<SubtitleInfo>> subtitleRetrieveFuture;

    //region Constructors

    protected AbstractTvDetailsComponent(LocaleText localeText,
                                         ImageService imageService,
                                         HealthService healthService,
                                         SettingsService settingsService,
                                         ApplicationEventPublisher eventPublisher,
                                         SubtitleService subtitleService, WatchedService watchedService) {
        super(localeText, imageService, healthService, settingsService);
        this.eventPublisher = eventPublisher;
        this.subtitleService = subtitleService;
        this.watchedService = watchedService;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeQualityList();
        initializeSubtitleList();
    }

    protected void initializeQualityList() {
        qualityList = new ListView<>();

        qualityList.setMaxWidth(200);
        qualityList.getItems().addListener((InvalidationListener) observable -> qualityList.setMaxHeight(50.0 * qualityList.getItems().size()));
        qualityList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onQualityChanged(newValue));
    }

    protected void initializeSubtitleList() {
        subtitleList = new ListView<>();

        subtitleList.setMaxWidth(200);
        subtitleList.getItems().addListener((InvalidationListener) observable -> subtitleList.setMaxHeight(50.0 * subtitleList.getItems().size()));
        subtitleList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onSubtitleChanged(newValue));
        subtitleList.setCellFactory(param -> new ListCell<>() {
            @Override
            protected void updateItem(SubtitleInfo item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    setText(item.getLanguage().getNativeName());
                } else {
                    setText(null);
                }
            }
        });
    }

    //endregion

    //region Functions

    /**
     * Reset the view details.
     */
    @Override
    protected void reset() {
        cancelSubtitleRetrievalIfNeeded();

        this.quality = null;
        this.subtitleRetrieveFuture = null;

        Platform.runLater(() -> {
            this.rating.setText(null);

            qualityList.getItems().clear();
            subtitleList.getItems().clear();
        });
    }

    @Override
    protected void load(T media) {
        super.load(media);

        loadRating();
    }

    /**
     * Retrieve the subtitles for the current media item.
     *
     * @return Returns the completable future of the subtitle retrieval.
     */
    protected abstract CompletableFuture<List<SubtitleInfo>> retrieveSubtitles();

    /**
     * Load the health information for the given quality.
     *
     * @param quality The media quality to retrieve the health from.
     */
    protected abstract void loadHealth(String quality);

    protected void loadSubtitles() {
        // check if another subtitle is already being retrieved
        cancelSubtitleRetrievalIfNeeded();

        subtitleRetrieveFuture = retrieveSubtitles();

        subtitleRetrieveFuture.whenComplete((subtitleInfos, throwable) -> {
            if (throwable == null) {
                this.subtitle = subtitleService.getDefaultOrInterfaceLanguage(subtitleInfos);

                Platform.runLater(() -> {
                    subtitleList.getItems().clear();
                    subtitleList.getItems().addAll(subtitleInfos);
                    subtitleList.getSelectionModel().select(this.subtitle);
                });
            } else if (isNotACancellationException(throwable)) {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    private void loadRating() {
        rating.setText(getRatingText());
    }

    private boolean isNotACancellationException(Throwable throwable) {
        return !CancellationException.class.isAssignableFrom(throwable.getClass());
    }

    private void onQualityChanged(String newValue) {
        if (StringUtils.isEmpty(newValue))
            return;

        quality = newValue;
        qualityButtonLabel.setText(newValue);
        loadHealth(newValue);
    }

    private void onSubtitleChanged(SubtitleInfo newValue) {
        this.subtitle = newValue;

        if (newValue != null) {
            subtitleLabel.setText(newValue.getLanguage().getNativeName());
        } else {
            subtitleLabel.setText(localeText.get(MediaMessage.SUBTITLE_NONE));
        }
    }

    private void cancelSubtitleRetrievalIfNeeded() {
        if (subtitleRetrieveFuture != null && !subtitleRetrieveFuture.isDone()) {
            log.trace("Cancelling current subtitle retrieval");
            subtitleRetrieveFuture.cancel(true);
        }
    }

    //endregion
}
