package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.controls.Stars;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.messages.MediaMessage;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.*;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.DisposableBean;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.task.TaskExecutor;
import org.springframework.util.Assert;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
public class MediaCardComponent implements Initializable, DisposableBean {
    private static final int POSTER_WIDTH = 134;
    private static final int POSTER_HEIGHT = 196;
    private static final String LIKED_STYLE_CLASS = "liked";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final List<ItemListener> listeners = new ArrayList<>();
    private final BooleanProperty favoriteProperty = new SimpleBooleanProperty();
    private final BooleanProperty watchedProperty = new SimpleBooleanProperty();
    private final LocaleText localeText;
    private final TaskExecutor taskExecutor;
    private final Media media;

    private Thread imageLoadingThread;

    @FXML
    private Pane posterItem;
    @FXML
    private BorderPane poster;
    @FXML
    private Label title;
    @FXML
    private Label year;
    @FXML
    private Label seasons;
    @FXML
    private Label ratingValue;
    @FXML
    private Icon favorite;
    @FXML
    private Stars ratingStars;

    public MediaCardComponent(Media media, LocaleText localeText, TaskExecutor taskExecutor, ItemListener... listeners) {
        this.media = media;
        this.localeText = localeText;
        this.taskExecutor = taskExecutor;
        this.listeners.addAll(asList(listeners));
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeImage();
        initializeText();
        initializeStars();
        initializeFavorite();
        initializeView();
    }

    /**
     * Set if this media card is liked by the user.
     *
     * @param value The favorite value.
     */
    public void setIsFavorite(boolean value) {
        favoriteProperty.set(value);
    }

    /**
     * Set if this media card is already watched by the user.
     *
     * @param value The watched/viewed value.
     */
    public void setIsWatched(boolean value) {
        watchedProperty.set(value);
    }

    /**
     * Add a listener to this instance.
     *
     * @param listener The listener to add.
     */
    public void addListener(ItemListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    @Override
    public void destroy() {
        Optional.ofNullable(imageLoadingThread)
                .ifPresent(Thread::interrupt);
    }

    private void initializeImage() {
        imageLoadingThread = new Thread(() -> {
            try {
                // set as default the poster holder image
                setBackgroundImage(new Image(new ClassPathResource("/images/posterholder.png").getInputStream(), POSTER_WIDTH, POSTER_HEIGHT, true, true));

                //try to load the actual image
                Optional.ofNullable(media.getImages())
                        .map(Images::getPoster)
                        .filter(e -> !e.equalsIgnoreCase("n/a"))
                        .ifPresent(e -> setBackgroundImage(new Image(e, POSTER_WIDTH, POSTER_HEIGHT, true, true)));
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });

        // run this on a separate thread for easier UI loading
        taskExecutor.execute(imageLoadingThread);
    }

    private void setBackgroundImage(Image image) {
        BackgroundSize size = new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, false, true);
        BackgroundImage backgroundImage = new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER, size);
        poster.setBackground(new Background(backgroundImage));
    }

    private void initializeText() {
        double rating = (double) media.getRating().getPercentage() / 10;

        title.setText(media.getTitle());
        year.setText(media.getYear());
        ratingValue.setText(rating + "/10");

        if (media instanceof Show) {
            Show show = (Show) media;
            String text = localeText.get(MediaMessage.SEASONS, show.getNumberOfSeasons());

            if (show.getNumberOfSeasons() > 1) {
                text += localeText.get(MediaMessage.PLURAL);
            }

            seasons.setText(text);
        }

        Tooltip.install(title, new Tooltip(media.getTitle()));
    }

    private void initializeStars() {
        ratingStars.setRating(media.getRating());
    }

    private void initializeFavorite() {
        switchFavorite(favoriteProperty.get());
        favoriteProperty.addListener((observable, oldValue, newValue) -> switchFavorite(newValue));
    }

    private void initializeView() {
        switchWatched(watchedProperty.get());
        watchedProperty.addListener((observable, oldValue, newValue) -> switchWatched(newValue));
    }

    private void switchFavorite(boolean isFavorite) {
        if (isFavorite) {
            favorite.getStyleClass().add(LIKED_STYLE_CLASS);
        } else {
            favorite.getStyleClass().remove(LIKED_STYLE_CLASS);
        }
    }

    private void switchWatched(boolean isWatched) {
        if (isWatched) {
            posterItem.getStyleClass().add(WATCHED_STYLE_CLASS);
        } else {
            posterItem.getStyleClass().remove(WATCHED_STYLE_CLASS);
        }
    }

    @FXML
    private void onWatchedClicked(MouseEvent event) {
        event.consume();
        boolean newValue = !watchedProperty.get();

        watchedProperty.set(newValue);
        synchronized (listeners) {
            listeners.forEach(e -> e.onWatchedChanged(media, newValue));
        }
    }

    @FXML
    private void onFavoriteClicked(MouseEvent event) {
        event.consume();
        boolean newValue = !favoriteProperty.get();

        favoriteProperty.set(newValue);
        synchronized (listeners) {
            listeners.forEach(e -> e.onFavoriteChanged(media, newValue));
        }
    }

    @FXML
    private void showDetails() {
        synchronized (listeners) {
            listeners.forEach(e -> e.onClicked(media));
        }
    }
}
