package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractDetailsComponent;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
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
    protected static final String SUBTITLE_STYLE_CLASS = "subtitle";
    protected static final String SUBTITLE_LOADING_STYLE_CLASS = "loading";
    protected static final String SUBTITLE_SUCCESS_STYLE_CLASS = "success";
    protected static final String SUBTITLE_FAILED_STYLE_CLASS = "failed";

    protected final ApplicationEventPublisher eventPublisher;
    protected final SubtitleService subtitleService;

    @FXML
    protected Overlay overlay;
    @FXML
    protected Icon subtitleStatus;
    @FXML
    protected Pane qualityButton;
    @FXML
    protected Label qualityButtonLabel;

    protected ListView<String> qualityList;
    protected String quality;
    protected SubtitleInfo subtitle;

    private CompletableFuture<List<SubtitleInfo>> subtitleRetrieveFuture;

    //region Constructors

    protected AbstractTvDetailsComponent(LocaleText localeText,
                                         ImageService imageService,
                                         HealthService healthService,
                                         SettingsService settingsService,
                                         ApplicationEventPublisher eventPublisher,
                                         SubtitleService subtitleService) {
        super(localeText, imageService, healthService, settingsService);
        this.eventPublisher = eventPublisher;
        this.subtitleService = subtitleService;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeQualityList();
    }

    protected void initializeQualityList() {
        qualityList = new ListView<>();

        qualityList.setMaxWidth(100);
        qualityList.getItems().addListener((InvalidationListener) observable -> qualityList.setMaxHeight(50.0 * qualityList.getItems().size()));
        qualityList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onQualityChanged(newValue));
    }

    //endregion

    //region Functions

    /**
     * Reset the view details.
     */
    protected void reset() {
        cancelSubtitleRetrievalIfNeeded();

        this.quality = null;
        this.subtitleRetrieveFuture = null;

        Platform.runLater(() -> {
            subtitleStatus.getStyleClass().removeIf(e -> !e.equals(SUBTITLE_STYLE_CLASS));
            qualityList.getItems().clear();
        });
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
        subtitleStatus.getStyleClass().add(SUBTITLE_LOADING_STYLE_CLASS);

        // check if another subtitle is already being retrieved
        cancelSubtitleRetrievalIfNeeded();

        subtitleRetrieveFuture = retrieveSubtitles();

        subtitleRetrieveFuture.whenComplete((subtitleInfos, throwable) -> {
            subtitleStatus.getStyleClass().remove(SUBTITLE_LOADING_STYLE_CLASS);

            if (throwable == null) {
                subtitle = subtitleService.getDefaultOrInterfaceLanguage(subtitleInfos);

                if (subtitle.isNone()) {
                    subtitleStatus.getStyleClass().add(SUBTITLE_FAILED_STYLE_CLASS);
                } else {
                    subtitleStatus.getStyleClass().add(SUBTITLE_SUCCESS_STYLE_CLASS);
                }
            } else if (isNotACancellationException(throwable)) {
                log.error(throwable.getMessage(), throwable);
                subtitleStatus.getStyleClass().add(SUBTITLE_FAILED_STYLE_CLASS);
            }
        });
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

    private void cancelSubtitleRetrievalIfNeeded() {
        if (subtitleRetrieveFuture != null && !subtitleRetrieveFuture.isDone()) {
            log.trace("Cancelling current subtitle retrieval");
            subtitleRetrieveFuture.cancel(true);
        }
    }

    //endregion
}
