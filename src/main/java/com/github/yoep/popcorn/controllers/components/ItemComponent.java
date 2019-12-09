package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.controls.Stars;
import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
public class ItemComponent implements Initializable {
    private static final int POSTER_WIDTH = 134;
    private static final int POSTER_HEIGHT = 196;

    private final List<ItemListener> listeners = new ArrayList<>();
    private final Media media;

    @FXML
    private BorderPane poster;
    @FXML
    private Label title;
    @FXML
    private Label year;
    @FXML
    private Label ratingValue;
    @FXML
    private Stars ratingStars;

    public ItemComponent(Media media) {
        this.media = media;
    }

    public ItemComponent(Media media, ItemListener... listeners) {
        this.media = media;
        this.listeners.addAll(asList(listeners));
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeImage();
        initializeText();
        initializeStars();
    }

    public void addListener(ItemListener listener) {
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    private void initializeImage() {
        try {
            Image image = Optional.ofNullable(media.getImages())
                    .map(Images::getPoster)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .map(e -> new Image(e, POSTER_WIDTH, POSTER_HEIGHT, true, true))
                    .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream(), POSTER_WIDTH, POSTER_HEIGHT, true, true));

            poster.setBackground(new Background(
                    new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER,
                            new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, false, true))));
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initializeText() {
        double rating = (double) media.getRating().getPercentage() / 10;

        title.setText(media.getTitle());
        year.setText(media.getYear());
        ratingValue.setText(rating + "/10");

        Tooltip.install(title, new Tooltip(media.getTitle()));
    }

    private void initializeStars() {
        ratingStars.setRating(media.getRating());
    }

    @FXML
    private void showDetails() {
        synchronized (listeners) {
            listeners.forEach(e -> e.onClicked(media));
        }
    }
}
