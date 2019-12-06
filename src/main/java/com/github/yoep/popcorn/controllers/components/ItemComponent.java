package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.font.controls.Icons;
import com.github.yoep.popcorn.providers.media.models.Images;
import com.github.yoep.popcorn.providers.media.models.Media;
import javafx.collections.ObservableList;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.*;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.text.StringEscapeUtils;
import org.springframework.core.io.ClassPathResource;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class ItemComponent implements Initializable {
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
    private HBox ratingStars;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeImage();
        initializeText();
        initializeStars();
    }

    private void initializeImage() {
        try {
            Image image = Optional.ofNullable(media.getImages())
                    .map(Images::getPoster)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .map(e -> new Image(e, 134, 196, true, true))
                    .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream(), 0, 196, true, true));

            poster.setBackground(new Background(
                    new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER,
                            new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, false, true))));
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initializeText() {
        String unescapedTitle = StringEscapeUtils.unescapeHtml4(media.getTitle());
        double rating = (double) media.getRating().getPercentage() / 10;

        title.setText(unescapedTitle);
        year.setText(media.getYear());
        ratingValue.setText(rating + "/10");

        Tooltip.install(title, new Tooltip(unescapedTitle));
    }

    private void initializeStars() {
        double rating = media.getRating().getPercentage() * 0.05;
        ObservableList<Node> starHolder = ratingStars.getChildren();

        for (int i = 0; i < 5; i++) {
            Node star;

            if (rating >= 0.75) {
                star = new Icon(Icons.STAR);
                star.getStyleClass().add("filled");
            } else if (rating < 0.75 && rating > 0.25) {
                Icon halfStar = new Icon(Icons.STAR_HALF);
                halfStar.getStyleClass().add("filled");
                Icon emptyStar = new Icon(Icons.STAR);
                emptyStar.getStyleClass().add("empty");
                StackPane stackpane = new StackPane(emptyStar, halfStar);
                stackpane.setAlignment(Pos.CENTER_LEFT);
                star = stackpane;
            } else {
                star = new Icon(Icons.STAR);
                star.getStyleClass().add("empty");
            }

            rating = --rating;
            starHolder.add(star);
        }
    }
}
