package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.config.properties.ProviderProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.TraktSettings;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.updater.UpdateState;
import com.github.yoep.popcorn.ui.updater.VersionInfo;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.AbstractFilterSectionController;
import com.github.yoep.popcorn.ui.view.controls.SearchField;
import com.github.yoep.popcorn.ui.view.controls.SearchListener;
import com.github.yoep.popcorn.ui.view.listeners.UpdateListener;
import com.github.yoep.popcorn.ui.view.services.UpdateSectionService;
import javafx.animation.Animation;
import javafx.animation.Transition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import javafx.util.Duration;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

@Slf4j
@ViewController
public class HeaderSectionController extends AbstractFilterSectionController implements Initializable {
    private final PopcornProperties properties;
    private final LocaleText localeText;
    private final PlatformProvider platformProvider;
    private final UpdateSectionService updateSectionService;

    private final Transition updateAvailableAnimation = createColorTransition();

    @FXML
    Pane headerPane;
    @FXML
    Pane titleBar;
    @FXML
    private ComboBox<Genre> genreCombo;
    @FXML
    private ComboBox<SortBy> sortByCombo;
    @FXML
    private SearchField search;
    @FXML
    private Icon watchlistIcon;
    @FXML
    private Icon torrentCollectionIcon;
    @FXML
    private Icon settingsIcon;
    @FXML
    private Icon aboutIcon;
    @FXML
    private Icon updateAvailableIcon;

    //region Constructors

    public HeaderSectionController(ApplicationEventPublisher eventPublisher, PopcornProperties properties, LocaleText localeText,
                                   SettingsService settingsService, PlatformProvider platformProvider, UpdateSectionService updateSectionService) {
        super(eventPublisher, settingsService);
        this.properties = properties;
        this.localeText = localeText;
        this.platformProvider = platformProvider;
        this.updateSectionService = updateSectionService;
    }

    //endregion

    //region Methods

    @EventListener(CloseSettingsEvent.class)
    public void onCloseSettings() {
        onSettingsClosed();
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeComboListeners();
        initializeSearchListener();
        initializeIcons();
        initializeSceneListener(headerPane);
        initializeTitleBar();
        initializeListeners();

        updateSectionService.updateAll();
    }

    private void initializeListeners() {
        updateSectionService.addListener(new UpdateListener() {
            @Override
            public void onUpdateInfoChanged(VersionInfo newValue) {
                // no-op
            }

            @Override
            public void onUpdateStateChanged(UpdateState newState) {
                onUpdateAvailableChanged(newState == UpdateState.UPDATE_AVAILABLE);
            }
        });
    }

    //endregion

    //region Functions

    @Override
    protected void clearSearch() {
        search.clear();
    }

    private void initializeComboListeners() {
        genreCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchGenre(newValue));
        sortByCombo.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> switchSortBy(newValue));
    }

    private void initializeSearchListener() {
        search.addListener(new SearchListener() {
            @Override
            public void onSearchValueChanged(String newValue) {
                eventPublisher.publishEvent(new SearchEvent(this, newValue));
            }

            @Override
            public void onSearchValueCleared() {
                eventPublisher.publishEvent(new SearchEvent(this, null));
            }
        });
    }

    private void initializeIcons() {
        var traktSettings = getSettings().getTraktSettings();

        watchlistIcon.setVisible(traktSettings.getAccessToken().isPresent());
        traktSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(TraktSettings.ACCESS_TOKEN_PROPERTY)) {
                watchlistIcon.setVisible(traktSettings.getAccessToken().isPresent());
            }
        });

        updateAvailableIcon.setVisible(false);

        Platform.runLater(updateAvailableAnimation::playFromStart);
    }

    private void initializeTitleBar() {
        var uiSettings = settingsService.getSettings().getUiSettings();

        if (uiSettings.isNativeWindowEnabled()) {
            headerPane.getChildren().remove(titleBar);
        }
    }

    private void onUpdateAvailableChanged(boolean updateAvailable) {
        platformProvider.runOnRenderer(() -> updateAvailableIcon.setVisible(updateAvailable));
    }

    @Override
    protected void updateGenres(Category category) {
        ProviderProperties providerProperties = properties.getProvider(category.getProviderName());
        List<Genre> genres = providerProperties.getGenres().stream()
                .map(e -> new Genre(e, localeText.get("genre_" + e)))
                .sorted()
                .collect(Collectors.toList());

        genreCombo.getItems().clear();
        genreCombo.getItems().addAll(genres);
        genreCombo.getSelectionModel().select(0);
    }

    @Override
    protected void updateSortBy(Category category) {
        ProviderProperties providerProperties = properties.getProvider(category.getProviderName());
        List<SortBy> sortBy = providerProperties.getSortBy().stream()
                .map(e -> new SortBy(e, localeText.get("sort-by_" + e)))
                .collect(Collectors.toList());

        sortByCombo.getItems().clear();
        sortByCombo.getItems().addAll(sortBy);
        sortByCombo.getSelectionModel().select(0);
    }

    private void switchGenre(Genre genre) {
        if (genre == null)
            return;

        log.trace("Genre is being changed to \"{}\"", genre);
        eventPublisher.publishEvent(new GenreChangeEvent(this, genre));
    }

    private void switchSortBy(SortBy sortBy) {
        if (sortBy == null)
            return;

        log.trace("SortBy is being changed to \"{}\"", sortBy);
        eventPublisher.publishEvent(new SortByChangeEvent(this, sortBy));
    }

    private void switchIcon(Icon icon) {
        removeAllActiveStates();

        activateItem(icon);
    }

    @Override
    protected void removeAllActiveStates() {
        super.removeAllActiveStates();

        // icons
        watchlistIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        torrentCollectionIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        settingsIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
        aboutIcon.getStyleClass().removeIf(e -> e.equals(ACTIVE_STYLE_CLASS));
    }

    private void onSettingsClosed() {
        Platform.runLater(() -> {
            removeAllActiveStates();
            activateItem(lastKnownSelectedCategory);
        });
    }

    private ApplicationSettings getSettings() {
        return settingsService.getSettings();
    }

    private Transition createColorTransition() {
        return new Transition() {
            {
                setCycleCount(Animation.INDEFINITE);
                setCycleDuration(Duration.seconds(2));
                setAutoReverse(true);
            }

            @Override
            protected void interpolate(double frac) {
                var color = Color.rgb(36, 104, 204);
                updateAvailableIcon.setTextFill(color.interpolate(Color.rgb(45, 150, 217), frac));
            }
        };
    }

    @FXML
    private void onCategoryClicked(MouseEvent event) {
        switchCategory((Label) event.getSource());
    }

    @FXML
    private void onGenreClicked() {
        genreCombo.show();
    }

    @FXML
    private void onSortClicked() {
        sortByCombo.show();
    }

    @FXML
    void onWatchlistClicked(MouseEvent event) {
        event.consume();
        switchIcon(watchlistIcon);
        eventPublisher.publishEvent(new ShowWatchlistEvent(this));
    }

    @FXML
    void onTorrentCollectionClicked(MouseEvent event) {
        event.consume();
        switchIcon(torrentCollectionIcon);
        eventPublisher.publishEvent(new ShowTorrentCollectionEvent(this));
    }

    @FXML
    void onSettingsClicked(MouseEvent event) {
        event.consume();
        switchIcon(settingsIcon);
        eventPublisher.publishEvent(new ShowSettingsEvent(this));
    }

    @FXML
    void onAboutClicked(MouseEvent event) {
        event.consume();
        switchIcon(aboutIcon);
        eventPublisher.publishEvent(new ShowAboutEvent(this));
    }

    @FXML
    void onUpdateAvailableClicked(MouseEvent event) {
        event.consume();
        switchIcon(updateAvailableIcon);
        eventPublisher.publishEvent(new ShowUpdateEvent(this));
    }

    //endregion
}
