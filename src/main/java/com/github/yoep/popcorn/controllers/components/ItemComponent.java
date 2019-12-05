package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.providers.media.models.Images;
import com.github.yoep.popcorn.providers.media.models.Media;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
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
    private ImageView poster;
    @FXML
    private Label title;
    @FXML
    private Label year;
    @FXML
    private Label ratingValue;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeImage();
        initializeText();
    }

    private void initializeImage() {
        try {
            Image image = Optional.ofNullable(media.getImages())
                    .map(Images::getPoster)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .map(e -> new Image(e, 0, 196, true, true))
                    .orElse(new Image(new ClassPathResource("/images/posterholder.png").getInputStream(), 0, 196, true, true));

            poster.setImage(image);
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
}
